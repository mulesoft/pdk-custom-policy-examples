// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use crate::generated::config::Config;
use agent_core::http_utils::with_no_timeout;
use anyhow::{anyhow, Result};

use pdk::hl::*;
use serde_json::from_slice;

async fn request_filter(request_state: RequestState) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;
    let header_handler = headers_state.handler();

    // Remove the timeout
    with_no_timeout(header_handler);

    Flow::Continue(())
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
    let filter = on_request(request_filter);
    launcher.launch(filter).await?;
    Ok(())
}
