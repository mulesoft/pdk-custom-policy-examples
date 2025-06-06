// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::policy_violation::PolicyViolations;

use crate::generated::config::Config;

pub const REJECT_MESSAGE: &str = r#"{ "error" : "forbidden string detected on the payload" }."#;

async fn request_filter(
    request_state: RequestState,
    policy_violations: &PolicyViolations,
    config: &Config,
) -> Flow<()> {
    // We specify that we'll treat the body as a stream instead of the normal `into_body_state` function.
    // When using this mode we lose the ability to modify the payload value, but we gain the capability
    // to process payloads whose size are bigger than the underlying buffer.
    let body_stream_state = request_state.into_body_stream_state().await;

    match config.search_mode.as_str() {
        "streamed" => streamed(body_stream_state, policy_violations, config).await,
        _ => buffered(body_stream_state, policy_violations, config).await,
    }
}

/// For each received chunk we search the accumulated buffer to check if we can abort
/// the request before the whole payload is received.
async fn streamed(
    body_stream_state: RequestBodyStreamState,
    policy_violations: &PolicyViolations,
    config: &Config,
) -> Flow<()> {
    let mut stream = body_stream_state.stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        // We read the payload by chunks
        buffer.append(&mut chunk.into_bytes()); // We accumulate the chunks in a local buffer
        let buffered_body = String::from_utf8_lossy(buffer.as_slice());

        // we check the buffered chunks for forbidden strings
        if config
            .forbidden_strings
            .iter()
            .any(|str| buffered_body.contains(str))
        {
            // We mark the request as policy violation.
            policy_violations.generate_policy_violation();
            return Flow::Break(Response::new(400).with_body(REJECT_MESSAGE));
        }
    }

    Flow::Continue(())
}

/// We collect all the chunks to do a single check once all the body is received.
async fn buffered(
    body_stream_state: RequestBodyStreamState,
    policy_violations: &PolicyViolations,
    config: &Config,
) -> Flow<()> {
    let mut stream = body_stream_state.stream();

    // PDK provides a helper method that awaits for all the chunks, so we don't have to manually
    // accumulate them if we need the whole body.
    let collect = stream.collect().await;
    let collected_body = String::from_utf8_lossy(collect.bytes());

    // we check the whole body for the forbidden string.
    if config
        .forbidden_strings
        .iter()
        .any(|str| collected_body.contains(str))
    {
        // We mark the request as policy violation.
        policy_violations.generate_policy_violation();
        return Flow::Break(Response::new(400).with_body(REJECT_MESSAGE));
    }

    Flow::Continue(())
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    violations: PolicyViolations,
) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;
    let filter = on_request(|rs| request_filter(rs, &violations, &config));
    launcher.launch(filter).await?;
    Ok(())
}
