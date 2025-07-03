// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::authentication::{Authentication, AuthenticationHandler};
use pdk::hl::*;
use pdk::script::{EvaluationError, HandlerAttributesBinding, Value};

use crate::generated::config::Config;

/// Wrap the script evaluation and transforms the result into an HTTP response.
async fn request_filter(
    request_state: RequestState,
    stream_properties: StreamProperties,
    authentication_data: Authentication,
    config: &Config,
) -> Flow<()> {
    match eval_script(
        request_state,
        stream_properties,
        authentication_data,
        config,
    )
    .await
    {
        Ok(val) => {
            let json: serde_json::Value = val.into();
            Flow::Break(
                Response::new(200)
                    .with_headers([("Content-Type".to_string(), "application/json".to_string())])
                    .with_body(format!(r#"{{"result":{json}}}"#)),
            )
        }
        Err(err) => Flow::Break(
            Response::new(400)
                .with_headers([("Content-Type".to_string(), "application/json".to_string())])
                .with_body(format!(r#"{{"error":"{err}}}""#)),
        ),
    }
}

async fn eval_script(
    request_state: RequestState,
    stream_properties: StreamProperties,
    authentication_data: Authentication,
    config: &Config,
) -> Result<Value, EvaluationError> {
    // We instantiate the expression evaluator
    let mut evaluator = config.expression.evaluator();

    // Bind the vars defined in the gcl.yaml to the evaluator. Only once all vars are bound to
    // the evaluator they will attempt to resolve the expression.
    evaluator.bind_vars("defaultId", "hardcoded");
    evaluator.bind_vars("version", "1");

    // After each bind, we can check if the evaluator has finished resolving the expression. If so
    // we can do an early return
    if evaluator.is_ready() {
        return evaluator.eval();
    }

    // Await the headers and bind them to the evaluator
    let headers_state = request_state.into_headers_state().await;
    evaluator.bind_attributes(&HandlerAttributesBinding::new(
        headers_state.handler(),
        &stream_properties,
    ));

    // Bind Authentication Object
    evaluator.bind_authentication(&authentication_data.authentication());

    if evaluator.is_ready() {
        return evaluator.eval();
    }

    // Await the body and bind them to the evaluator
    let body_state = headers_state.into_body_state().await;
    evaluator.bind_payload(&body_state);

    evaluator.eval()
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
    let filter = on_request(|rs, stream, auth| request_filter(rs, stream, auth, &config));
    launcher.launch(filter).await?;
    Ok(())
}
