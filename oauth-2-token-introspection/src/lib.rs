// Copyright 2025 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::logger::{debug, error};
use pdk::token_introspection::{IntrospectionError, IntrospectionResult, ParsedToken, ScopesValidator, TokenValidator, TokenValidatorBuilder, ValidationError};
use crate::generated::config::Config;

const AUTHORIZATION_HEADER: &str = "authorization";
const X_AGW_PREFIX: &str = "x-agw";
const CONTENT_TYPE_JSON: &str = "application/json; charset=UTF-8";
const WWW_AUTHENTICATE_OAUTH2: &str = "Bearer realm=\"OAuth2 Introspection Client Realm\"";

// Exposes token claims as HTTP headers with `x-agw-` prefix.
fn expose_claims(handler: &dyn HeadersHandler, parsed_token: &ParsedToken) {
    for (key, value) in parsed_token.properties().iter() {
        let header_name = format!("{}-{}", X_AGW_PREFIX, key.replace(' ', "-"));
        if let Some(s) = value.as_str() {
            handler.set_header(&header_name, s);
        } else if let Some(b) = value.as_bool() {
            handler.set_header(&header_name, &b.to_string());
        } else if let Some(n) = value.as_f64() {
            handler.set_header(&header_name, &n.to_string());
        }
    }
}

// Creates JSON error response with given status code and message.
fn json_error_response(status: u32, message: &str) -> Response {
    Response::new(status)
        .with_headers(vec![(
            "Content-Type".to_string(),
            CONTENT_TYPE_JSON.to_string(),
        )])
        .with_body(serde_json::json!({"error": message}).to_string())
}

// Creates 401 Unauthorized response with WWW-Authenticate header.
fn unauthorized_response(message: &str) -> Response {
    Response::new(401)
        .with_headers(vec![
            (
                "WWW-Authenticate".to_string(),
                WWW_AUTHENTICATE_OAUTH2.to_string(),
            ),
            ("Content-Type".to_string(), CONTENT_TYPE_JSON.to_string()),
        ])
        .with_body(serde_json::json!({"error": message}).to_string())
}

// Extracts token, validates via introspection, validates contract.
async fn do_request_filter(
    handler: &dyn HeadersHandler,
    validator: &TokenValidator,
) -> Result<IntrospectionResult, IntrospectionError> {
    let auth_header = handler.header(AUTHORIZATION_HEADER).unwrap_or_else(|| String::from(""));
    let token = auth_header.split_whitespace().collect::<Vec<_>>()[1];

    debug!("Token extracted successfully (len={})", token.len());

    // Validate token.
    let result = validator
        .validate(&token)
        .await?;

    Ok(result)
}

// Maps IntrospectionError to HTTP Response.
fn introspection_error_response(error: &IntrospectionError) -> Response {
    match error {
        IntrospectionError::RequestFailed(_) => unauthorized_response("Unauthorized"),
        IntrospectionError::HttpError { status, body } => match status {
            400 => json_error_response(500, body),
            403 => json_error_response(403, body),
            _ => unauthorized_response(body),
        },
        IntrospectionError::ParseError(_) => json_error_response(
            500,
            "Unexpected error processing Authentication Server response",
        ),
        IntrospectionError::Validation(v) => match v {
            ValidationError::TokenExpired => unauthorized_response("Token has expired."),
            ValidationError::TokenRevoked => unauthorized_response("Token has been revoked."),
            ValidationError::InvalidScopes => {
                json_error_response(403, "The required scopes are not authorized")
            }
            _ => unauthorized_response("Token validation failed"),
        },
        _ => json_error_response(500, "Unexpected introspection error"),
    }
}

async fn request_filter(
    request_state: RequestState,
    expose_headers: bool,
    validator: &TokenValidator,
) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;
    let handler = headers_state.handler();

    match do_request_filter(handler, validator).await {
        Ok(success) => {
            if expose_headers {
                expose_claims(handler, &success.token);
            }
            Flow::Continue(())
        }
        Err(error) => {
            Flow::Break(introspection_error_response(&error))
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(bytes): Configuration,
    validator_builder: TokenValidatorBuilder,
) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let mut validator_instance = validator_builder
        .new("token-cache")
        .with_path(config.introspection_path)
        .with_authorization_value(config.authorization_value)
        .with_expires_in_attribute(config.expires_in_attribute)
        .with_max_token_ttl(config.validated_token_ttl as i64)
        .with_timeout_ms(config.authentication_timeout as u64)
        .with_max_cache_entries(config.max_cache_entries as usize)
        .with_service(config.introspection_service.clone());

    if let Some(scopes) = config.scopes {
        let scopes_v = scopes
            .split_whitespace()
            .map(|scope| scope.to_string())
            .collect();
        if !scopes.is_empty() {
            let validator = match config.scope_validation_criteria.to_uppercase().as_str() {
                "AND" => ScopesValidator::all(scopes_v),
                _ => ScopesValidator::any(scopes_v),
            };
            validator_instance = validator_instance.with_scopes_validator(validator);
        }
    }

    let validator = validator_instance.build().map_err(|e| {
        error!("Failed to build TokenValidator: {e}");
        LaunchError {}
    })?;

    let expose_headers = config.expose_headers;

    let filter = on_request(|rs| request_filter(rs, expose_headers, &validator));
    launcher.launch(filter).await?;
    Ok(())
}