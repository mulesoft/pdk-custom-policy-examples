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

    loop {
        let (current_stats, version) = match storage.get::<RequestStats>(&key).await {
            Ok(Some((stats, ver))) => (stats, ver),
            _ => (RequestStats::default(), "0".to_string()),
        };

        let mut new_stats = current_stats;
        new_stats.count += 1;
        new_stats.last_request = now;

        match storage.store(&key, &StoreMode::Cas(version), &new_stats).await {
            Ok(_) => return new_stats,
            Err(_) => {
                // CAS failed, retry with updated version
                continue;
            }
        }
    }
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
    let mut status = 200;
    let mut body = None;
    
    if state.handler().header("x-stats").is_some() {
        let all_stats = get_all_stats(storage, &config.namespace).await;
        let stats_json = serde_json::to_string(&all_stats).unwrap();
        headers.push(("x-all-stats".to_string(), stats_json));
    } else if state.handler().header("x-reset-stats").is_some() {
        let _ = storage.delete_all().await;
        headers.push(("x-stats-reset".to_string(), "true".to_string()));
    } else {
        match get_client_id(&state).await {
            Some(client_id) => {
                let stats = update_request_stats(storage, &client_id, &config.namespace).await;
                headers.extend(vec![
                    ("x-request-count".to_string(), stats.count.to_string()),
                    ("x-client-id".to_string(), client_id.clone()),
                    ("x-last-request".to_string(), stats.last_request.to_string()),
                ]);
            }
            None => {
                status = 400;
                body = Some("Missing client identification header (x-client-id)");
            }
        }
    }

    let mut response = Response::new(status).with_headers(headers);
    if let Some(body_content) = body {
        response = response.with_body(body_content);
    }
    
    Flow::Break(response)
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