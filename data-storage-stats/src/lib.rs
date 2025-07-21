// Copyright 2023 Salesforce, Inc. All rights reserved.
use data_storage_lib::{DataStorage, DataStorageBuilder, StoreMode};
use pdk::hl::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_namespace")]
    namespace: String,
}

fn default_namespace() -> String {
    "request-stats".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestStats {
    count: u64,
    last_request: u64,
}

impl Default for RequestStats {
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

async fn get_client_id(state: &RequestHeadersState) -> Option<String> {
    if let Some(client_id) = state.handler().header("x-client-id") {
        if !client_id.is_empty() {
            return Some(client_id.to_string());
        }
    }
    
    None
}

async fn update_request_stats<T: DataStorage>(
    storage: &T,
    client_id: &str,
    namespace: &str,
) -> RequestStats {
    let key = format!("{}:{}", namespace, client_id);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut stats = match storage.get::<RequestStats>(&key).await {
        Ok(Some((existing_stats, _))) => existing_stats,
        _ => RequestStats::default(),
    };

    stats.count += 1; // Update stats
    stats.last_request = now;

    // Store updated stats directly as the lib handles serialization
    let _ = storage.store(&key, &StoreMode::Always, &stats).await;

    stats
}

async fn get_all_stats<T: DataStorage>(storage: &T, namespace: &str) -> HashMap<String, RequestStats> {
    let mut all_stats = HashMap::new();
    
    if let Ok(keys) = storage.get_keys().await {
        for key in keys {
            if key.starts_with(&format!("{}:", namespace)) {
                if let Ok(Some((stats, _))) = storage.get::<RequestStats>(&key).await {
                    let client_id = key.replace(&format!("{}:", namespace), "");
                    all_stats.insert(client_id, stats);
                }
            }
        }
    }

    all_stats
}

async fn request_filter(
    state: RequestHeadersState,
    storage: &impl DataStorage,
    config: &Config,
) -> Flow<()> {
    let mut headers = vec![];
    
    if state.handler().header("x-stats").is_some() {
        let all_stats = get_all_stats(storage, &config.namespace).await;
        let stats_json = serde_json::to_string(&all_stats).unwrap();
        headers.push(("x-all-stats".to_string(), stats_json));
        return Flow::Break(Response::new(200).with_headers(headers));
    }
    
    if state.handler().header("x-reset-stats").is_some() {
        let _ = storage.delete_all().await;
        headers.push(("x-stats-reset".to_string(), "true".to_string()));
        return Flow::Break(Response::new(200).with_headers(headers));
    }
    
    let client_id = match get_client_id(&state).await {
        Some(id) => id,
        None => {
            return Flow::Break(Response::new(400).with_body("Missing client identification header (x-client-id)"));
        }
    };

    let stats = update_request_stats(storage, &client_id, &config.namespace).await;

    headers.extend(vec![
        ("x-request-count".to_string(), stats.count.to_string()),
        ("x-client-id".to_string(), client_id.clone()),
        ("x-last-request".to_string(), stats.last_request.to_string()),
    ]);

    Flow::Break(Response::new(200).with_headers(headers))
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    store_builder: DataStorageBuilder,
    Configuration(configuration): Configuration,
) -> anyhow::Result<()> {
    let config: Config = serde_json::from_slice(&configuration)?;
    let storage = store_builder.local(config.namespace.clone());

    launcher
        .launch(on_request(|request| request_filter(request, &storage, &config)))
        .await?;
    Ok(())
} 