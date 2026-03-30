// Copyright 2023 Salesforce, Inc. All rights reserved.
use anyhow::Result;

use pdk::cors;
use pdk::hl::*;
use pdk::logger;

pub mod convert;
pub mod generated;
use generated::config::Config;

async fn request_filter(
    state: RequestHeadersState,
    cors: &cors::Cors<'_>,
) -> Flow<Vec<(String, String)>> {
    logger::info!("Validating CORS on request.");

    // Determine what kind of request is incoming.
    match cors.check_headers(state.handler().headers().as_slice()) {
        Ok(check) => match check.response_type() {
            // A preflight request must return a 200 OK.
            cors::ResponseType::Preflight => {
                logger::info!("Preflight CORS response.");
                Flow::Break(Response::new(200).with_headers(check.into_headers()))
            }

            // A main request must continue.
            cors::ResponseType::Main => {
                logger::info!("Main CORS response.");
                Flow::Continue(check.into_headers())
            }
        },

        // A validation problem ocurred. Must return early.
        Err(message) => {
            logger::debug!("Request finished with the following error {message}");
            Flow::Break(Response::new(200))
        }
    }
}

async fn response_filter(state: ResponseHeadersState, data: RequestData<Vec<(String, String)>>) {
    logger::info!("Applying CORS on response.");

    // Take the validation headers from the request.
    let RequestData::Continue(headers_to_add) = data else {
        logger::info!("There are no headers to add.");

        return;
    };

    // Add all the validation headers into the response.
    for (name, value) in headers_to_add.iter() {
        logger::info!("Adding header {name} = {value}");
        state.handler().set_header(name, value);
    }
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(config): Configuration,
) -> Result<(), LaunchError> {
    logger::info!("Deserializing new configuration.");
    let config: Config = match serde_json::from_slice(&config) {
        Ok(config) => config,
        Err(err) => {
            logger::error!("Problem deserializing configuration: {err}");
            return Ok(());
        }
    };

    logger::info!("Translating deserialized configuration.");

    // Translate the filter configuration into a CORS Configuration.
    let cors_config = match config.into_cors() {
        Ok(config) => config,
        Err(err) => {
            logger::error!("Problem translating configuration: {err}");
            return Ok(());
        }
    };

    let cors = cors::Cors::new(&cors_config);

    let filter = on_request(|rs| request_filter(rs, &cors)).on_response(response_filter);

    // Launch the filter.
    launcher.launch(filter).await
}

#[cfg(test)]
mod tests {
    use pdk_unit::{UnitHttpRequest, UnitTestBuilder};
    use serde_json::json;

    fn config_with_origin(origin: &str) -> String {
        json!({
            "publicResource": false,
            "supportCredentials": false,
            "originGroups": [
                {
                    "name": "group1",
                    "origins": [origin],
                    "allowedMethods": [
                        {"methodName": "GET", "allowed": true}
                    ],
                    "headers": [],
                    "exposedHeaders": [],
                    "accessControlMaxAge": 300.0
                }
            ]
        })
        .to_string()
    }

    #[test]
    fn request_from_allowed_origin_passes() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config_with_origin("https://example.com"))
            .with_entrypoint(crate::configure);

        let response = tester
            .request_full(UnitHttpRequest::get().with_header("origin", "https://example.com"));

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn preflight_request_returns_response() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config_with_origin("https://example.com"))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::options()
                .with_header("origin", "https://example.com")
                .with_header("access-control-request-method", "GET"),
        );

        assert!(response.status_code() == 200 || response.status_code() == 204);
    }

    #[test]
    fn request_without_origin_passes() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config_with_origin("https://example.com"))
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 200);
    }
}
