// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::ip_filter::IpFilter;

use crate::generated::config::Config;

// Apply IP filter to specific IP header
async fn request_filter(
    request_state: RequestState,
    ip_filter: &IpFilter,
    ip_header: &str,
) -> Flow<()> {
    let headers = request_state.into_headers_state().await;

    let client_ip = headers.handler().header(ip_header);

    match client_ip {
        // If IP is allowed, continue
        Some(ip) if ip_filter.is_allowed(&ip) => Flow::Continue(()),

        // If IP is not allowed, break
        _ => Flow::Break(Response::new(403).with_body("Forbidden IP!")),
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

    // Save allowed IPs from config
    let ip_values: Vec<&str> = config.ips.iter().map(|s| s.as_str()).collect();

    // Create IP filter with allowed IPs
    let ip_filter = IpFilter::allow(&ip_values)?;

    // Create filter with IP filter and header name
    let filter = on_request(|rs| request_filter(rs, &ip_filter, &config.ip_header));

    launcher.launch(filter).await?;

    Ok(())
}
