// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use std::str::FromStr;

use pdk::hl::*;

use crate::generated::config::Config;

// This function executes an outbound requests and forwards the response to the caller.
async fn request_filter(client: HttpClient, service: &Service) -> Response {
    match client.request(service).get().await {
        Ok(response) => Response::new(response.status_code()).with_body(response.body()),
        Err(e) => Response::new(503).with_body(e.to_string()),
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    // Create a service pointing to the upstream that requires custom tls configuration.
    let service = Service::from(
        &config.service.name,
        &config.service.namespace,
        Uri::from_str(&config.service.url)?,
    );

    let filter = on_request(|client| request_filter(client, &service));
    launcher.launch(filter).await?;
    Ok(())
}
