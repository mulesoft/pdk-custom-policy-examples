// Copyright 2023 Salesforce, Inc. All rights reserved.
mod decorator;
mod generated;
mod openai;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::logger;
use serde_json::json;

use crate::generated::config::Config;

use decorator::CompletionDecorator;

/// Decorates a chat request.
async fn decorate_request(
    headers_state: RequestHeadersState,
    decorator: &CompletionDecorator<'_>,
) -> Result<(), (u32, &'static str)> {
    let headers_handler = headers_state.handler();

    // Removing old content length header before manipulating body
    headers_handler.remove_header("content-length");

    // Move to the body state.
    let body_state = headers_state.into_body_state().await;
    let body_handler = body_state.handler();

    // Extract body
    let input_body = body_handler.body();

    // Deserialize payload
    let payload = serde_json::from_slice(&input_body)
        .map_err(|_| (400, "Unable to deserialize JSON payload."))?;

    // Decorate payload
    let decorated_payload = decorator.decorate(payload);

    let output_body = serde_json::to_vec(&decorated_payload).map_err(|e| {
        logger::error!("Unable to serialize decorated body: {e:?}");
        (500, "Internal error.")
    })?;

    body_handler.set_body(&output_body).map_err(|e| {
        logger::error!("Unable to set new body: {e:?}");
        (400, "Payload too long.")
    })
}

/// Decorates the input chat request.
async fn request_filter(
    headers_state: RequestHeadersState,
    decorator: &CompletionDecorator<'_>,
) -> Flow<()> {
    logger::info!("Processing incoming request.");

    match decorate_request(headers_state, decorator).await {
        Ok(_) => {
            logger::info!("Request decorated.");
            Flow::Continue(())
        }
        Err((status_code, error)) => Flow::Break(
            Response::new(status_code)
                .with_body(json!({ "error": error }).to_string())
                .with_headers([("Content-Type".to_string(), "application/json".to_string())]),
        ),
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    logger::info!("Initializing AI prompt decorator policy.");

    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {err}",
            String::from_utf8_lossy(&bytes),
        )
    })?;

    let decorator = CompletionDecorator::from_config(&config);

    let filter = on_request(|request_state| request_filter(request_state, &decorator));

    logger::info!("Starting filters.");

    launcher.launch(filter).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{
        TraceBackend, UnitHttpMessage, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder,
    };
    use serde_json::json;
    use std::rc::Rc;

    fn config(prepend: serde_json::Value, append: serde_json::Value) -> String {
        json!({ "prepend": prepend, "append": append }).to_string()
    }

    fn base_body() -> String {
        json!({
            "model": "gpt-4",
            "messages": [
                {"role": "user", "content": "Hello!"}
            ]
        })
        .to_string()
    }

    fn messages_from_backend(backend: &TraceBackend<UnitHttpResponse>) -> serde_json::Value {
        let req = backend.next().unwrap();
        let body: serde_json::Value = serde_json::from_slice(req.body()).unwrap();
        body["messages"].clone()
    }

    #[test]
    fn clean_request_passes_through() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config(json!([]), json!([])))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::post().with_body(base_body()));

        assert_eq!(response.status_code(), 200);
        let messages = messages_from_backend(&backend);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello!");
    }

    #[test]
    fn prepend_messages_are_added() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config(
                json!([{"role": "system", "content": "You are a helpful assistant."}]),
                json!([]),
            ))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::post().with_body(base_body()));

        assert_eq!(response.status_code(), 200);
        let messages = messages_from_backend(&backend);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are a helpful assistant.");
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[1]["content"], "Hello!");
    }

    #[test]
    fn append_messages_are_added() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config(
                json!([]),
                json!([{"role": "user", "content": "Remember to be concise."}]),
            ))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::post().with_body(base_body()));

        assert_eq!(response.status_code(), 200);
        let messages = messages_from_backend(&backend);
        let messages = messages.as_array().unwrap();
        assert_eq!(
            messages.last().unwrap()["content"],
            "Remember to be concise."
        );
    }

    #[test]
    fn prepend_and_append_both_applied() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config(
                json!([{"role": "system", "content": "System prompt."}]),
                json!([{"role": "user", "content": "Closing message."}]),
            ))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::post().with_body(base_body()));

        assert_eq!(response.status_code(), 200);
        let messages = messages_from_backend(&backend);
        let messages = messages.as_array().unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["content"], "System prompt.");
        assert_eq!(messages[1]["content"], "Hello!");
        assert_eq!(messages[2]["content"], "Closing message.");
    }

    #[test]
    fn invalid_json_body_returns_400() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config(json!([]), json!([])))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::post().with_body("not valid json"));

        assert_eq!(response.status_code(), 400);
    }

    #[test]
    fn multiple_prepend_messages_preserve_order() {
        let backend = Rc::new(TraceBackend::new(UnitHttpResponse::new(200)));
        let mut tester = UnitTestBuilder::default()
            .with_config(config(
                json!([
                    {"role": "system", "content": "First."},
                    {"role": "system", "content": "Second."}
                ]),
                json!([]),
            ))
            .with_backend(Rc::clone(&backend))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::post().with_body(base_body()));

        assert_eq!(response.status_code(), 200);
        let messages = messages_from_backend(&backend);
        let messages = messages.as_array().unwrap();
        assert_eq!(messages[0]["content"], "First.");
        assert_eq!(messages[1]["content"], "Second.");
        assert_eq!(messages[2]["content"], "Hello!");
    }
}
