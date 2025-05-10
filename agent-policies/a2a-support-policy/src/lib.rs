// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use crate::generated::config::Config;

use agent_core::a2a::{is_valid_method, valid_methods, valid_request};
use agent_core::anypoint::{HttpPlatformClient, PlatformClient};
use agent_core::http_utils::{
    APPLICATION_JSON, CONTENT_LENGTH_HEADER, CONTENT_TYPE_HEADER, POST_METHOD,
};
use agent_core::json_rpc::{JsonRpcRequest, RpcError};

use anyhow::{anyhow, Result};
use pdk::hl::*;
use pdk::logger;
use pdk::metadata::Metadata;
use pdk::script::PayloadBinding;
use serde_json::{from_slice, Error, Value};


async fn request_filter(request_state: RequestState, config: &Config) -> Flow<AgentCard> {
    let header_state = request_state.into_headers_state().await;
    let path = header_state.path();
    let card = path.to_lowercase().ends_with(config.card_path.as_str());
    let agent_card = AgentCard { is_card: card };
    if config.verify_schema {
        match header_state.method().as_str() {
            POST_METHOD => {
                let maybe_content_type = header_state.handler().header(CONTENT_TYPE_HEADER);
                if let Some(content_type) = maybe_content_type {
                    let mime: mime::Mime = content_type.parse().unwrap();
                    if mime.subtype() == mime::JSON {
                        // This should be the messages endpoint
                        let request_body_state = header_state.into_body_state().await;
                        let body_bytes = request_body_state.as_bytes();
                        let result_json_request =
                            from_slice::<JsonRpcRequest<'_>>(body_bytes.as_slice());
                        match result_json_request {
                            Ok(json_request) => {
                                if !is_valid_method(json_request.method) {
                                    let error_message = RpcError::invalid_methods(
                                        json_request.method.to_string(),
                                        valid_methods(),
                                    );
                                    create_rpc_error_response(&error_message)
                                } else {
                                    let result = valid_request(json_request);
                                    if let Err(error) = result {
                                        let rpc_error = RpcError::invalid_param(error.to_string());
                                        create_rpc_error_response(&rpc_error)
                                    } else {
                                        Flow::Continue(agent_card)
                                    }
                                }
                            }
                            Err(err) => {
                                let error_message = RpcError::invalid_json(err.to_string());
                                create_rpc_error_response(&error_message)
                            }
                        }
                    } else {
                        Flow::Continue(agent_card)
                    }
                } else {
                    Flow::Continue(agent_card)
                }
            }
            _ => Flow::Continue(agent_card),
        }
    } else {
        Flow::Continue(agent_card)
    }
}

fn create_rpc_error_response(error_message: &RpcError) -> Flow<AgentCard> {
    let error = serde_json::to_string(&error_message).unwrap_or_default();
    Flow::Break(
        Response::new(400)
            .with_headers(vec![(
                CONTENT_TYPE_HEADER.to_string(),
                APPLICATION_JSON.to_string(),
            )])
            .with_body(error),
    )
}

async fn response_filter(
    response_state: ResponseState,
    consumer_url: &str,
    request_data: RequestData<AgentCard>,
) {
    match request_data {
        RequestData::Continue(d) => {
            if d.is_card {
                let state = response_state.into_headers_state().await;

                let maybe_content_type = state.handler().header(CONTENT_TYPE_HEADER);
                match maybe_content_type {
                    None => {}
                    Some(content_type) => {
                        let mime: mime::Mime = content_type.parse().unwrap();
                        if mime.subtype() == mime::JSON {
                            //as we are updating
                            state.handler().remove_header(CONTENT_LENGTH_HEADER);
                            let body_state = state.into_body_state().await;
                            let json_bytes = body_state.as_bytes();
                            let result: Result<Value, Error> = from_slice(&json_bytes);
                            if let Ok(mut json_value) = result {
                                if let Value::Object(ref mut map) = json_value {
                                    map.insert(
                                        "url".to_string(),
                                        Value::String(consumer_url.to_string()),
                                    );
                                }
                                // Convert back to Vec<u8>
                                let updated_json = serde_json::to_vec(&json_value)
                                    .expect("failed to serialize json");

                                body_state
                                    .handler()
                                    .set_body(updated_json.as_slice())
                                    .expect("failed to set body");
                            }
                        }
                    }
                }
            }
        }
        RequestData::Break => {}
        RequestData::Cancel => {}
    }
}

#[derive(Debug)]
struct AgentCard {
    is_card: bool,
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    client: HttpClient,
    metadata: Metadata,
) -> Result<()> {
    let config: Config = from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let url = match &config.consumer_url {
        None => {
            let result = resolve_consumer_url(&client, &metadata).await;
            match result {
                Ok(url) => url,
                Err(err) => {
                    logger::error!(
                        "Unable to resolve consumer url from the platform.\nCause: {}",
                        err
                    );
                    return Err(err);
                }
            }
        }
        Some(consumer_url) => sanitized_base_path(consumer_url),
    };

    let filter = on_request(|rs| request_filter(rs, &config));
    let filter = filter.on_response(|rs, request_data| response_filter(rs, &url, request_data));
    launcher.launch(filter).await?;
    Ok(())
}

async fn resolve_consumer_url(client: &HttpClient, metadata: &Metadata) -> Result<String> {
    let anypoint_client = HttpPlatformClient::new(client, metadata);
    let login_response = anypoint_client.login().await?;
    let token = login_response.get_token();
    let apim_login_response = anypoint_client.login_apim(token).await?;
    let maybe_consumer_url = anypoint_client
        .consumer_url(apim_login_response.get_token())
        .await?;
    let consumer_url = maybe_consumer_url
        .expect("Unable to resolve consumer url and it was not specified in the config");
    Ok(sanitized_base_path(consumer_url.as_str()))
}

///
/// Card url needs to end with / lets verify it
///
fn sanitized_base_path(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}
