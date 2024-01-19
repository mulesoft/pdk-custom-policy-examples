// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use itertools::Itertools;
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

/// This function takes the query parameters defined in the config and forwards them as headers.
fn update_headers(
    state: &RequestHeadersState,
    query_params: &HashMap<String, String>,
    config: &Config,
) {
    // Forward the query parameters present in the config as headers.
    for query in &config.query {
        state.handler().set_header(
            format!("x-query-{query}").as_str(),
            query_params
                .get(query)
                .map(String::as_str)
                .unwrap_or(&UNDEFINED),
        )
    }
}

/// This function recreated the query parameters by removing the specified query parameter and adding
/// new ones that indicate which parameters were removed.
fn modify_query_params(
    state: &RequestHeadersState,
    query_params: &HashMap<String, String>,
    config: &Config,
) -> Result<()> {
    let path = state.path();

    // Create a fake url and append the path.
    let mut url = Url::parse("http://fake_base").and_then(|url| url.join(path.as_str()))?;

    // We create a new scope to isolate the mutable handling of the url.
    {
        let mut new_query = url.query_pairs_mut();
        // We clean all existing query parameters.
        new_query.clear();

        // We recreate the query parameters ordered alphabetically.
        for (key, val) in query_params.iter().sorted() {
            // We avoid coping the ones we want to remove, and add a new value for the "removed" parameter
            if config.query.contains(key) {
                new_query.append_pair("removed", key);
            // We copy the other key only query params.
            } else if val.is_empty() {
                new_query.append_key_only(key);
            // We copy the other query params.
            } else {
                new_query.append_pair(key, val);
            }
        }
        // We finish editing the params.
        new_query.finish();
    }

    // We create the new path string appending the query string if present.
    let new_path = url
        .query()
        .map(|query| format!("{}?{query}", url.path()))
        .unwrap_or_else(|| url.path().to_string());

    // We replace the path pseudo header to set the new query parameters.
    state.handler().set_header(":path", new_path.as_str());

    Ok(())
}

/// This filter takes the query parameters and forwards them as headers.
async fn inner_request_filter(
    request_state: RequestState,
    config: &Config,
) -> Result<(), Response> {
    let state = request_state.into_headers_state().await;

    // Read the query parameters from the headers state.
    let query_params = query_params(&state).map_err(|err| {
        Response::new(401).with_body(format!("Failed to parse query params. Cause: {err}"))
    })?;

    // Manipulate the query parameters of the request.
    modify_query_params(&state, &query_params, config).map_err(|err| {
        Response::new(503).with_body(format!("Failed to modify the query params. Cause: {err}"))
    })?;

    // Forward the desired query parameters to the headers.
    update_headers(&state, &query_params, config);

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
