// Copyright 2025 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::logger::{debug, error, trace, warn};
use pdk::token_introspection::{IntrospectionError, IntrospectionResult, ParsedToken, ScopesValidator, TokenValidator, TokenValidatorBuilder, ValidationError};
use crate::generated::config::Config;

const AUTHORIZATION_HEADER: &str = "authorization";
const ACCESS_TOKEN_PARAM: &str = "access_token";
const PATH_HEADER: &str = ":path";
const X_AGW_PREFIX: &str = "x-agw";
const CONTENT_TYPE_JSON: &str = "application/json; charset=UTF-8";
const WWW_AUTHENTICATE_OAUTH2: &str = "Bearer realm=\"OAuth2 Introspection Client Realm\"";
const BEARER: &str = "bearer";

// Token extraction errors.
#[derive(Debug)]
enum ExtractionError {
    AccessTokenNotProvided,
    InvalidAuthorizationHeader,
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionError::AccessTokenNotProvided => {
                write!(f, "Access token was not provided")
            }
            ExtractionError::InvalidAuthorizationHeader => {
                write!(f, "Authorization header is invalid")
            }
        }
    }
}

// Extracts access token from Authorization header or query parameter.
//
// - If the Authorization header is present, it must be a valid Bearer header (otherwise fail).
// - Otherwise, use the access_token query parameter when present.
// - If neither is provided, fail.
fn extract_token(
    query_token: Option<&str>,
    auth_header: Option<&str>,
) -> Result<String, ExtractionError> {
    let query_token = query_token.filter(|s| !s.trim().is_empty());
    let auth_header = auth_header.filter(|s| !s.trim().is_empty());

    if let Some(header) = auth_header {
        return extract_bearer(header).ok_or(ExtractionError::InvalidAuthorizationHeader);
    }

    if let Some(token) = query_token {
        return Ok(token.to_string());
    }

    Err(ExtractionError::AccessTokenNotProvided)
}

// Extracts Bearer token from Authorization header.
// Returns Some(token) if valid Bearer, None otherwise.
fn extract_bearer(header: &str) -> Option<String> {
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() == 2 && parts[0].to_lowercase() == BEARER {
        Some(parts[1].to_string())
    } else {
        None
    }
}

// Policy error types.
enum PolicyError {
    // Token extraction failed.
    Extraction(ExtractionError),
    // Token introspection/validation failed.
    Introspection(IntrospectionError),
}

// Result of successful token validation.
struct TokenSuccess {
    // The introspection result containing parsed token and access token.
    result: IntrospectionResult,
}

// Extracts a query parameter value from a URL path.
fn extract_query_param(path: &str, name: &str) -> Option<String> {
    url::Url::parse(&format!("http://fake_base{path}"))
        .ok()?
        .query_pairs()
        .find_map(|(key, value)| (key == name).then(|| value.to_string()))
}

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

fn get_query_token(handler: &dyn HeadersHandler, auth_header: Option<String>) -> Option<String> {
    if auth_header
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .is_some()
    {
        None
    } else {
        handler
            .header(PATH_HEADER)
            .as_deref()
            .and_then(|p| extract_query_param(p, ACCESS_TOKEN_PARAM))
    }
}

// Extracts token, validates via introspection, validates contract.
async fn do_request_filter(
    handler: &dyn HeadersHandler,
    validator: &TokenValidator,
) -> Result<TokenSuccess, PolicyError> {
    // Extract token from request
    let auth_header = handler.header(AUTHORIZATION_HEADER);
    let query_token = get_query_token(handler, auth_header.clone());

    debug!(
        "Token extraction - auth_header present: {}, query_token present: {}",
        auth_header
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .is_some(),
        query_token.is_some()
    );

    let token = extract_token(query_token.as_deref(), auth_header.as_deref())
        .map_err(PolicyError::Extraction)?;

    debug!("Token extracted successfully (len={})", token.len());

    // Validate token.
    let result = validator
        .validate(&token)
        .await
        .map_err(PolicyError::Introspection)?;

    Ok(TokenSuccess { result })
}

// Maps PolicyError to HTTP Response.
fn error_response(error: &PolicyError) -> Response {
    match error {
        PolicyError::Extraction(e) => json_error_response(400, &e.to_string()),
        PolicyError::Introspection(e) => introspection_error_response(e),
    }
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

// Logs error message based on the PolicyError.
fn log_error(error: &PolicyError) {
    match error {
        PolicyError::Extraction(e) => {
            trace!("Error extracting token parameter: {e} (ErrorCode: FED-400)")
        }
        PolicyError::Introspection(e) => match e {
            IntrospectionError::RequestFailed(e) => {
                error!("Error trying to send authorize token request. Reason: {e}")
            }
            IntrospectionError::HttpError { status, body } => {
                warn!("Invalid status code {status} while processing auth response: {body}")
            }
            IntrospectionError::ParseError(e) => {
                error!("Error creating token from authentication server response: {e}")
            }
            IntrospectionError::Validation(v) => {
                warn!("Invalid token: {v}")
            }
            _ => error!("Unexpected introspection error: {e}"),
        },
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
                expose_claims(handler, &success.result.token);
            }
            Flow::Continue(())
        }
        Err(error) => {
            log_error(&error);
            Flow::Break(error_response(&error))
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