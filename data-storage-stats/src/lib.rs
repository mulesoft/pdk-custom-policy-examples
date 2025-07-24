// Copyright 2023 Salesforce, Inc. All rights reserved.

mod constants;

use constants::*;
use data_storage_lib::{DataStorage, DataStorageBuilder, StoreMode};
use pdk::hl::*;
use pdk::logger;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration parameters for the data storage stats policy.
#[derive(Deserialize, Clone)]
struct Config {
    /// Namespace used to isolate data between different policy instances.
    #[serde(default)]
    namespace: String,

    /// Storage type to use: "local" for in-memory storage or "remote" for distributed storage.
    #[serde(default)]
    storage_type: String,

    /// Time-to-live for stored items in seconds (used only for remote storage).
    #[serde(default)]
    ttl_seconds: u32,

    /// Maximum number of retries for CAS operations.
    max_retries: u32,
}

impl Default for Config {
    /// Default configuration values.
    fn default() -> Self {
        Self {
            namespace: DEFAULT_NAMESPACE.to_string(),
            storage_type: DEFAULT_STORAGE_TYPE.to_string(),
            ttl_seconds: DEFAULT_TTL_SECONDS,
            max_retries: 0, // This will be overridden by the mandatory field
        }
    }
}

/// Statistics for a client's request activity.
#[derive(Serialize, Deserialize, Debug)]
struct RequestStats {
    /// Total number of requests made by client.
    count: u64,

    /// Unix timestamp (in seconds) of client's last request.
    last_request: u64,
}

impl Default for RequestStats {
    /// Creates a new RequestStats instance with count 0 and current timestamp.
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            count: 0,
            last_request: now,
        }
    }
}

/// Extracts the client ID from request headers.
async fn get_client_id(state: &RequestHeadersState) -> Option<String> {
    if let Some(client_id) = state.handler().header(CLIENT_ID_HEADER) {
        // Validate that the client ID is not empty
        if !client_id.is_empty() {
            return Some(client_id.to_string());
        }
    }

    None
}

/// Updates request statistics using CAS operations with retry logic.
async fn update_request_stats<T: DataStorage>(
    storage: &T,
    client_id: &str,
    namespace: &str,
    max_retries: u32,
) -> Result<RequestStats, String> {
    let key = format!("{}:{}", namespace, client_id);

    // Get current timestamp for the last_request field
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut retry_count = 0;

    loop {
        if retry_count >= max_retries {
            logger::error!(
                "Storage operation failed after {} retries for client {}",
                max_retries,
                client_id
            );
            return Err(format!(
                "Storage operation failed after {} retries for client {}",
                max_retries, client_id
            ));
        }

        // Try to get existing stats or create new ones
        let (current_stats, mode) = match storage.get::<RequestStats>(&key).await {
            Ok(Some((stats, ver))) => (stats, StoreMode::Cas(ver)),
            Ok(None) => (RequestStats::default(), StoreMode::Absent),
            Err(e) => {
                logger::warn!(
                    "Storage error for key: {}, retry: {} - {:?}",
                    key,
                    retry_count,
                    e
                );
                retry_count += 1;
                continue;
            }
        };

        let mut new_stats = current_stats;
        new_stats.count += 1;
        new_stats.last_request = now;

        // Attempt CAS operation to atomically update the stats
        match storage.store(&key, &mode, &new_stats).await {
            Ok(_) => return Ok(new_stats),
            Err(_) => {
                retry_count += 1;
                continue;
            }
        }
    }
}

/// Retrieves all client statistics from storage.
async fn get_all_stats<T: DataStorage>(
    storage: &T,
    namespace: &str,
) -> HashMap<String, RequestStats> {
    let mut all_stats = HashMap::new();

    // Get all keys from storage
    if let Ok(keys) = storage.get_keys().await {
        for key in keys {
            // Only process keys that belong to our namespace
            if key.starts_with(&format!("{}:", namespace)) {
                // Retrieve stats for this client
                if let Ok(Some((request_stats, _))) = storage.get::<RequestStats>(&key).await {
                    // Extract client ID by removing namespace prefix
                    let client_id = key.replace(&format!("{}:", namespace), "");
                    all_stats.insert(client_id, request_stats);
                }
            }
        }
    }

    logger::info!("Retrieved {} client stats", all_stats.len());
    all_stats
}

/// Main request filter that handles stats operations.
///
/// Admin operations:
/// - GET /stats: Return all client statistics as JSON
/// - DELETE /stats: Reset all statistics and return confirmation
async fn request_filter<T: DataStorage>(
    state: RequestHeadersState,
    storage: Rc<T>,
    config: Config,
) -> Flow<()> {
    // Route request based on path for RESTful API design
    let path = state.path();
    let method = state.method();

    if path == "/stats" && method == "GET" {
        // Admin operation: GET /stats - return stats as JSON in response body
        let all_stats = get_all_stats(&*storage, &config.namespace).await;
        let stats_json = serde_json::to_string_pretty(&all_stats).unwrap();
        Flow::Break(
            Response::new(200)
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])
                .with_body(stats_json),
        )
    } else if path == "/stats" && method == "DELETE" {
        // Admin operation: DELETE /stats - reset all stats and return confirmation
        let _ = storage.delete_all().await;
        let response_body = serde_json::json!({
            "message": "All statistics have been reset successfully",
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        let response_json = serde_json::to_string_pretty(&response_body).unwrap();
        Flow::Break(
            Response::new(200)
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])
                .with_body(response_json),
        )
    } else {
        // Client operation: any other path - update stats and continue to backend
        match get_client_id(&state).await {
            Some(client_id) => {
                match update_request_stats(
                    &*storage,
                    &client_id,
                    &config.namespace,
                    config.max_retries,
                )
                .await
                {
                    Ok(_) => Flow::Continue(()),
                    Err(error_msg) => {
                        logger::error!("Failed to update stats: {}", error_msg);
                        Flow::Break(Response::new(500).with_body(error_msg))
                    }
                }
            }
            None => {
                logger::warn!("Missing client identification header");
                Flow::Break(
                    Response::new(400)
                        .with_body("Missing client identification header (x-client-id)"),
                )
            }
        }
    }
}

/// Policy entrypoint that configures storage and launches request handler.
#[entrypoint]
async fn configure(
    launcher: Launcher,
    store_builder: DataStorageBuilder,
    Configuration(configuration): Configuration,
) -> anyhow::Result<()> {
    let config: Config = serde_json::from_slice(&configuration)?;

    if config.storage_type == LOCAL_STORAGE {
        logger::info!("Using LOCAL storage");
        let local = Rc::new(store_builder.local(config.namespace.clone()));
        launcher
            .launch(on_request(move |request| {
                request_filter(request, local.clone(), config.clone())
            }))
            .await?;
    } else if config.storage_type == REMOTE_STORAGE {
        logger::info!("Using REMOTE storage");
        let remote =
            Rc::new(store_builder.remote(config.namespace.clone(), config.ttl_seconds * 1000));
        launcher
            .launch(on_request(move |request| {
                request_filter(request, remote.clone(), config.clone())
            }))
            .await?;
    } else {
        return Err(anyhow::anyhow!(
            "Invalid storage type: {}",
            config.storage_type
        ));
    }

    Ok(())
}
