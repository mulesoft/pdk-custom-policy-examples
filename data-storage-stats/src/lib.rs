// Copyright 2023 Salesforce, Inc. All rights reserved.

mod generated;

use pdk::data_storage::{DataStorage, DataStorageBuilder, StoreMode};
use pdk::hl::*;
use pdk::logger;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::generated::config::Config;

// HTTP header for client identification (input only)
pub const CLIENT_ID_HEADER: &str = "x-client-id";

// Storage type values
pub const REMOTE_STORAGE: &str = "remote";
pub const LOCAL_STORAGE: &str = "local";

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
fn get_client_id(state: &RequestHeadersState) -> Option<String> {
    if let Some(client_id) = state.handler().header(CLIENT_ID_HEADER) {
        // Validate that the client ID is not empty
        if !client_id.is_empty() {
            return Some(client_id.to_string());
        }
    }

    None
}

/// Updates request statistics using CAS operations with retry logic.
async fn update_request_stats(
    storage: &impl DataStorage,
    client_id: &str,
    namespace: &str,
    max_retries: u32,
) -> Result<RequestStats, String> {
    let key = format!("{namespace}:{client_id}");

    // Get current timestamp for the last_request field
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut retry_count = 0;

    loop {
        if retry_count >= max_retries {
            logger::error!(
                "Storage operation failed after {max_retries} retries for client {client_id}"
            );
            return Err(format!(
                "Storage operation failed after {max_retries} retries for client {client_id}"
            ));
        }

        // Try to get existing stats or create new ones
        let (current_stats, mode) = match storage.get::<RequestStats>(&key).await {
            Ok(Some((stats, ver))) => (stats, StoreMode::Cas(ver)),
            Ok(None) => (RequestStats::default(), StoreMode::Absent),
            Err(e) => {
                logger::warn!("Storage error for key: {key}, retry: {retry_count} - {e:?}");
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
async fn get_all_stats(
    storage: &impl DataStorage,
    namespace: &str,
) -> HashMap<String, RequestStats> {
    let mut all_stats = HashMap::new();

    // Get all keys from storage
    if let Ok(keys) = storage.get_keys().await {
        let namespace_prefix = format!("{namespace}:");
        for key in keys {
            // Only process keys that belong to our namespace
            if key.starts_with(&namespace_prefix) {
                // Retrieve stats for this client
                if let Ok(Some((request_stats, _))) = storage.get::<RequestStats>(&key).await {
                    // Extract client ID by removing namespace prefix
                    let client_id = key.replace(&namespace_prefix, "");
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
async fn request_filter(
    state: RequestHeadersState,
    storage: &impl DataStorage,
    config: &Config,
) -> Flow<()> {
    // Route request based on path for RESTful API design
    let path = state.path();
    let method = state.method();

    if path == "/stats" && method == "GET" {
        // Admin operation: GET /stats - return stats as JSON in response body
        let all_stats = get_all_stats(storage, &config.namespace).await;
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
        match get_client_id(&state) {
            Some(client_id) => {
                let max_retries = config.max_retries as u32;
                match update_request_stats(storage, &client_id, &config.namespace, max_retries)
                    .await
                {
                    Ok(_) => Flow::Continue(()),
                    Err(error_msg) => {
                        logger::error!("Failed to update stats: {error_msg}");
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

    let storage_type = &config.storage_type;
    let namespace = &config.namespace;

    match storage_type.as_str() {
        LOCAL_STORAGE => {
            logger::info!("CONFIG: Local storage");
            let local = store_builder.local(namespace);
            launcher
                .launch(on_request(|request| {
                    request_filter(request, &local, &config)
                }))
                .await?;
        }
        REMOTE_STORAGE => {
            logger::info!("CONFIG: Remote storage");
            let ttl_seconds = config.ttl_seconds as u32;
            let remote = store_builder.remote(namespace, ttl_seconds * 1000);
            launcher
                .launch(on_request(|request| {
                    request_filter(request, &remote, &config)
                }))
                .await?;
        }
        _ => {
            return Err(anyhow::anyhow!("Invalid storage type: {storage_type}"));
        }
    }

    Ok(())
}
