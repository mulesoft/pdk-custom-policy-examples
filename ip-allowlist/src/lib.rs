// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use crate::generated::config::Config;
use pdk::authentication::{Authentication, AuthenticationHandler};
use pdk::hl::*;
use pdk::ip_filter::IpFilter;
use pdk::script::{HandlerAttributesBinding, Script, Value};

async fn request_filter(
    request_state: RequestState,
    stream_properties: StreamProperties,
    auth: Authentication,
    ip_filter: &IpFilter,
    script: &Script,
) -> Flow<()> {
    let headers = request_state.into_headers_state().await;

    let mut evaluator = script.evaluator();
    let binding = HandlerAttributesBinding::new(headers.handler(), &stream_properties);
    evaluator.bind_attributes(&binding);
    evaluator.bind_authentication(&auth.authentication());

    let client_ip = match evaluator.eval() {
        Ok(Value::String(ip)) => ip.split(',').next().unwrap_or(&ip).trim().to_string(),
        _ => return Flow::Break(Response::new(403).with_body("Forbidden")),
    };

    if ip_filter.is_allowed(&client_ip) {
        Flow::Continue(())
    } else {
        Flow::Break(Response::new(403).with_body("Forbidden"))
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

    let ip_filter = IpFilter::allow(&config.ips).map_err(|e| anyhow!("Invalid IP: {e}"))?;

    let script = config.ip_expression;

    let filter = on_request(|rs, sp, auth| request_filter(rs, sp, auth, &ip_filter, &script));
    launcher.launch(filter).await?;
    Ok(())
}
