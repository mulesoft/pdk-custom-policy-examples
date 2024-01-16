// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::Result;

use crate::generated::config::Config;
use pdk::hl::*;
use pdk::logger;
use pdk::script::{DefaultBindings, TryFromValue};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Defines the custom errors to handle them in an unified way
pub enum FilterError {
    Unexpected,
    NoToken,
    InactiveToken,
    ExpiredToken,
    NotYetActive,
    ClientError(HttpClientError),
    NonParsableIntrospectionBody(serde_json::Error),
}

/// This struct parses the response of the rfc7662 compliant introspection endpoint
#[derive(Deserialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    pub exp: Option<u64>,
    pub nbf: Option<u64>,
}

/// Sends the request to the introspection endpoint and parse the response
async fn introspect_token(
    token: &str,
    config: &Config,
    client: HttpClient,
) -> Result<IntrospectionResponse, FilterError> {
    // Encodes the token for the request payload
    let body =
        serde_urlencoded::to_string([("token", token)]).map_err(|_| FilterError::Unexpected)?;

    logger::debug!("sending body: {}", body);

    // Sets the content type and add the configured authentication header
    let headers = vec![
        ("content-type", "application/x-www-form-urlencoded"),
        ("Authorization", config.authorization.as_str()),
    ];

    logger::debug!("About to call the introspection service.");
    // Executes the request with the configured upstream and await the response
    let response = client
        .request(&config.oauth_service)
        .headers(headers)
        .body(body.as_bytes())
        .post()
        .await
        .map_err(FilterError::ClientError)?;

    // Parses the response from the backend
    if response.status_code() == 200 {
        serde_json::from_slice(response.body()).map_err(FilterError::NonParsableIntrospectionBody)
    } else {
        let status = response.status_code();
        let body = String::from_utf8_lossy(response.body());
        logger::debug!("status is {} and body is {}", status, body);
        Err(FilterError::InactiveToken)
    }
}

/// Parses the token, sends it to the introspection endpoint, and validate the response
async fn do_filter(
    request: RequestHeadersState,
    config: &Config,
    client: HttpClient,
    eval: DefaultBindings,
) -> Result<(), FilterError> {
    // Extracts the token from the request
    let mut evaluator = eval.evaluator(&config.token_extractor);
    evaluator.bind_headers(request.handler());

    let token: String = evaluator
        .eval()
        .and_then(TryFromValue::try_from_value)
        .map_err(|_| FilterError::NoToken)?;

    // Sends the token to the introspection endpoint
    let response = introspect_token(token.as_str(), config, client).await?;

    // Obtains the current time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| FilterError::Unexpected)?
        .as_secs();

    // Validates if the token is active
    if !response.active {
        return Err(FilterError::InactiveToken);
    }

    // Validates if the token has expired
    if response.exp.map(|exp| now > exp).unwrap_or_default() {
        return Err(FilterError::ExpiredToken);
    }

    // Validates if the token has started its validity period
    if response.nbf.map(|nbf| now < nbf).unwrap_or_default() {
        return Err(FilterError::NotYetActive);
    }

    // Validation succeeded!
    Ok(())
}

/// Generates a standard early response that indicates the token validation failed
fn unauthorized_response() -> Flow<()> {
    Flow::Break(Response::new(401).with_headers(vec![(
        "WWW-Authenticate".to_string(),
        "Bearer realm=\"oauth2\"".to_string(),
    )]))
}

/// Generates a standard early response that indicates that there was an unexpected error
fn server_error_response() -> Flow<()> {
    Flow::Break(Response::new(500))
}

/// Defines a filter function that works as a wrapper for the real filter function that enables simplified error handling
async fn request_filter(
    state: RequestState,
    client: HttpClient,
    config: &Config,
    eval: DefaultBindings,
) -> Flow<()> {
    let state = state.into_headers_state().await;

    match do_filter(state, config, client, eval).await {
        Ok(_) => Flow::Continue(()),
        Err(err) => match err {
            FilterError::Unexpected => {
                logger::warn!("Unexpected error occurred while processing the request.");
                server_error_response()
            }
            FilterError::NoToken => {
                logger::debug!("No authorization token was provided.");
                unauthorized_response()
            }
            FilterError::InactiveToken => {
                logger::debug!("Token is marked as inactive by the introspection endpoint.");
                unauthorized_response()
            }
            FilterError::ExpiredToken => {
                logger::debug!("Expiration time on the token has been exceeded.");
                unauthorized_response()
            }
            FilterError::NotYetActive => {
                logger::debug!(
                    "Token is not yet valid, since time set in the nbf claim has not been reached."
                );
                unauthorized_response()
            }
            FilterError::ClientError(err) => {
                logger::warn!(
                    "Error sending the request to the introspection endpoint. {:?}.",
                    err
                );
                server_error_response()
            }
            FilterError::NonParsableIntrospectionBody(err) => {
                logger::warn!(
                    "Error parsing the response from the introspection endpoint. {}.",
                    err
                );
                server_error_response()
            }
        },
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).unwrap();
    launcher
        .launch(on_request(|request, client, eval| {
            request_filter(request, client, &config, eval)
        }))
        .await?;
    Ok(())
}
