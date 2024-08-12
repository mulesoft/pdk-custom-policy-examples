// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use crate::generated::config::Config;
use pdk::hl::grpc::*;
use pdk::hl::*;
use pdk::logger;
use pdk::script::{HandlerAttributesBinding, TryFromValue};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::{AuthRequest, AuthResponse};

include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

struct AuthClient {
    upstream: Service,
    client: GrpcClient,
}

impl AuthClient {
    fn new(upstream: Service, client: GrpcClient) -> Self {
        Self { upstream, client }
    }

    async fn check(&self, request: AuthRequest) -> Result<AuthResponse, GrpcClientError> {
        logger::info!("Validating Authentication.");

        let response = self
            .client
            .request(&self.upstream)
            .service("AuthService")
            .method("Check")
            .protobuf()
            .send(&request)
            .await?
            .into_inner();

        Ok(response)
    }
}

/// Defines the custom errors to handle them in an unified way
pub enum FilterError {
    Unexpected,
    NoToken,
    InactiveToken,
    ExpiredToken,
    NotYetActive,
    ClientError(GrpcClientError),
}

/// Parses the token, sends it to the introspection endpoint, and validate the response
async fn do_filter(
    request: RequestHeadersState,
    config: &Config,
    client: &AuthClient,
) -> Result<(), FilterError> {
    // Extracts the token from the request
    let mut evaluator = config.token_extractor.evaluator();
    evaluator.bind_attributes(&HandlerAttributesBinding::partial(request.handler()));

    let token: String = evaluator
        .eval()
        .and_then(TryFromValue::try_from_value)
        .map_err(|_| FilterError::NoToken)?;

    let request = AuthRequest {
        token,
        ..Default::default()
    };

    // Sends the token to the introspection endpoint
    let response = client
        .check(request)
        .await
        .map_err(FilterError::ClientError)?;

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
    if response.exp.filter(|exp| now > *exp).is_some() {
        return Err(FilterError::ExpiredToken);
    }

    // Validates if the token has started its validity period
    if response.nbf.filter(|nbf| now < *nbf).is_some() {
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
async fn request_filter(state: RequestState, client: &AuthClient, config: &Config) -> Flow<()> {
    let state = state.into_headers_state().await;

    match do_filter(state, config, client).await {
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
        },
    }
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    client: GrpcClient,
) -> Result<()> {
    logger::info!("Initializing gRPC OAuth 2.0 policy");

    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    logger::info!("gRPC OAuth 2.0 policy configuration parsed");

    let client = AuthClient::new(config.oauth_service.clone(), client);

    launcher
        .launch(on_request(|request| {
            request_filter(request, &client, &config)
        }))
        .await?;

    Ok(())
}
