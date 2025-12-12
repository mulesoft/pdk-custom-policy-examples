// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use crate::generated::config::Config;
use pdk::hl::*;
use pdk::ip_filter::IpFilter;

async fn request_filter(request_state: RequestState, ip_filter: &IpFilter) -> Flow<()> {
    let headers = request_state.into_headers_state().await;
    // Get client IP from x-forwarded-for header (first IP if multiple)
    let client_ip = headers
        .handler()
        .header("x-forwarded-for")
        .map(|h| h.split(',').next().unwrap_or(&h).trim().to_string());
    match client_ip {
        // If the IP is allowed, continue the flow
        Some(ip) if ip_filter.is_allowed(&ip) => Flow::Continue(()),
        // If the IP is not allowed, break the flow
        _ => Flow::Break(Response::new(403).with_body("Forbidden")),
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
    // Create allowlist from configured IPs
    let ip_filter = IpFilter::allow(&config.ips).map_err(|e| anyhow!("Invalid IP: {e}"))?;

    let filter = on_request(|rs| request_filter(rs, &ip_filter));
    launcher.launch(filter).await?;
    Ok(())
}
