// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use crate::generated::config::Config;
use agent_core::http_utils::with_no_timeout;
use anyhow::{anyhow, Result};

use eventsource::event::Event;
use pdk::hl::*;
use pdk::logger;
use serde_json::from_slice;

// This Filter detects event stream request and sets
async fn request_filter(request_state: RequestState) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;
    // Remove the timeout
    let header_handler = headers_state.handler();
    with_no_timeout(header_handler);
    Flow::Continue(())
}

fn transform_event(mut m: Event, config: &Config) -> Option<Event> {
    let data = &m.data;
    if data.trim().starts_with("{") {
        //is a json response if not
        Some(m)
    } else {
        logger::info!("Not a json\n{}", m.to_string());
        if let Some(base_url) = &config.base_url {
            m.data = format!("{}{}", base_url, data);
        } else {
            logger::info!("Not a json\n{}", m.to_string());
        }
        Some(m)
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let _config: Config = from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;
    let filter = on_request(|rs| request_filter(rs));
    launcher.launch(filter).await?;
    Ok(())
}
