// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::ip_filter::IpFilter;

use crate::generated::config::Config;

// Apply IP filters to specific IP header
async fn request_filter(
    request_state: RequestState,
    allow_filter: &Option<IpFilter>,
    block_filter: &Option<IpFilter>,
    ip_header: &str,
) -> Flow<()> {
    let headers = request_state.into_headers_state().await;

    let Some(ip) = headers.handler().header(ip_header) else {
        return Flow::Continue(());
    };

    if let Some(filter) = block_filter {
        if !filter.is_allowed(&ip) {
            return Flow::Break(Response::new(403).with_body("Blocked IP!"));
        }
    }

    if let Some(filter) = allow_filter {
        if !filter.is_allowed(&ip) {
            return Flow::Break(Response::new(403).with_body("IP not allowed!"));
        }
    }

    Flow::Continue(())
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

    // Create allow filter if ipsAllowed is configured
    let allow_filter = match &config.ips_allowed {
        Some(ips) if !ips.is_empty() => {
            let ip_values: Vec<&str> = ips.iter().map(|s| s.as_str()).collect();
            Some(IpFilter::allow(&ip_values)?)
        }
        _ => None,
    };

    // Create block filter if ipsBlocked is configured (IPs that match are blocked)
    let block_filter = match &config.ips_blocked {
        Some(ips) if !ips.is_empty() => {
            let ip_values: Vec<&str> = ips.iter().map(|s| s.as_str()).collect();
            Some(IpFilter::block(&ip_values)?)
        }
        _ => None,
    };

    // Create filter with both IP filters and header name
    let filter =
        on_request(|rs| request_filter(rs, &allow_filter, &block_filter, &config.ip_header));

    launcher.launch(filter).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{UnitHttpRequest, UnitTestBuilder};
    use serde_json::json;

    #[test]
    fn blocked_ip_is_rejected() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "ipHeader": "x-forwarded-for",
                    "ipsBlocked": ["192.168.1.1"]
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        let response =
            tester.request(UnitHttpRequest::get().with_header("x-forwarded-for", "192.168.1.1"));

        assert_eq!(response.status_code(), 403);
    }

    #[test]
    fn allowed_ip_passes_through() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "ipHeader": "x-forwarded-for",
                    "ipsAllowed": ["10.0.0.0/8"]
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        let response =
            tester.request(UnitHttpRequest::get().with_header("x-forwarded-for", "10.0.0.1"));

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn ip_not_in_allowlist_is_rejected() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "ipHeader": "x-forwarded-for",
                    "ipsAllowed": ["10.0.0.0/8"]
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        let response =
            tester.request(UnitHttpRequest::get().with_header("x-forwarded-for", "192.168.1.1"));

        assert_eq!(response.status_code(), 403);
    }

    #[test]
    fn missing_ip_header_passes_through() {
        let mut tester = UnitTestBuilder::default()
            .with_config(
                json!({
                    "ipHeader": "x-forwarded-for",
                    "ipsBlocked": ["192.168.1.1"]
                })
                .to_string(),
            )
            .with_entrypoint(crate::configure);

        let response = tester.request(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 200);
    }
}
