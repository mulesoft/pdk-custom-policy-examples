// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use std::collections::HashMap;

use pdk::hl::*;
use url::Url;

use crate::generated::config::Config;

/// Value to be used if the query parameter is absent from the request.
const UNDEFINED: &str = "Undefined";

/// Function that wraps the logic of the policy to simplify the error handling.
async fn request_filter(request_state: RequestState, config: &Config) -> Flow<()> {
    match inner_request_filter(request_state, config).await {
        // If we could do the transformation the request reaches the backend.
        Ok(()) => Flow::Continue(()),
        // We send an early response otherwise.
        Err(resp) => Flow::Break(resp),
    }
}

/// This function extracts the query parameters from the RequestHeadersState's path and returns them
/// as a map.
fn query_params(state: &RequestHeadersState) -> Result<HashMap<String, String>> {
    // Read the request's path.
    let path = state.path();

    // Create a fake url and append the path.
    let url = Url::parse("http://fake_base").and_then(|url| url.join(path.as_str()))?;

    // Extract the query pairs from the url.
    Ok(url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}

/// This filter takes the query parameters and forwards them as headers.
async fn inner_request_filter(
    request_state: RequestState,
    config: &Config,
) -> Result<(), Response> {
    let headers_state = request_state.into_headers_state().await;

    // read the query parameters from the headers state.
    let query_params = query_params(&headers_state).map_err(|err| {
        Response::new(401).with_body(format!("Failed to parse query params. Cause: {err}"))
    })?;

    // Forward the query parameters present in the config as headers.
    for query in &config.query {
        headers_state.handler().set_header(
            format!("x-query-{query}").as_str(),
            query_params
                .get(query)
                .map(String::as_str)
                .unwrap_or(&UNDEFINED),
        )
    }

    Ok(())
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

    let filter = on_request(|rs| request_filter(rs, &config));
    launcher.launch(filter).await?;
    Ok(())
}
