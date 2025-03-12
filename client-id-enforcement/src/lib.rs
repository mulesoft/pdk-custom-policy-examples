// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use std::sync::LazyLock;

use futures::join;
use generated::config::Config;
use pdk::authentication::Authentication;
use pdk::authentication::AuthenticationData;
use pdk::authentication::AuthenticationHandler;
use pdk::contracts::basic_auth_credentials;
use pdk::contracts::ContractValidator;
use pdk::hl::*;
use pdk::logger;
use serde_json::json;
use timer::Clock;

const AUTH_HEADER: &str = "WWW-Authenticate";
const AUTH_HEADER_VALUE: &str = r#"Basic realm="mule-realm""#;
const CONTENT_TYPE_HEADER: &str = "Content-Type";
const CONTENT_TYPE_HEADER_VALUE: &str = "application/json";

static RESPONSE_HEADERS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    [
        (AUTH_HEADER, AUTH_HEADER_VALUE),
        (CONTENT_TYPE_HEADER, CONTENT_TYPE_HEADER_VALUE),
    ]
    .iter()
    .map(|(header, value)| (header.to_string(), value.to_string()))
    .collect()
});

fn unauthorized_response(message: &str, status_code: u32) -> Response {
    logger::debug!("Invalid client credentials, rejecting request");
    Response::new(status_code)
        .with_headers(RESPONSE_HEADERS.clone())
        .with_body(json!({ "error": message }).to_string())
}

/// Validates user authentication from Basic Auth credentials backed by the [ContractValidator]. 
async fn authentication_filter(
    state: RequestHeadersState,
    authentication: Authentication,
    validator: &ContractValidator,
) -> Flow<()> {

    // Extract Basic Auth credentials from request
    let (client_id, client_secret) = match basic_auth_credentials(&state) {
        Ok(credentials) => credentials,
        Err(e) => {
            logger::info!("Invalid credentials: {e}");
            return Flow::Break(unauthorized_response("Invalid credentials", 401));
        }
    };

    // Validate authentication for the current user
    let validation = validator.authenticate(&client_id, &client_secret);

    let client_data = match validation {
        Ok(client_data) => client_data,
        Err(e) => {
            logger::info!("Invalid authentication: {e}");
            return Flow::Break(unauthorized_response("Invalid authentication", 403));
        }
    };

    // Update the current authentication data
    if let Some(auth) = authentication.authentication() {
        authentication.set_authentication(Some(&AuthenticationData {
            client_id: Some(client_data.client_id),
            client_name: Some(client_data.client_name),
            ..auth
        }));
    }

    Flow::Continue(())
}

/// Validates user authorization from Basic Auth credentials backed by the [ContractValidator]. 
async fn authorization_filter(
    state: RequestHeadersState,
    authentication: Authentication,
    validator: &ContractValidator,
) -> Flow<()> {
    // Extract Basic Auth credentials from request and dismiss client secret
    let (id, _) = match basic_auth_credentials(&state) {
        Ok(credentials) => credentials,
        Err(e) => {
            logger::info!("Invalid credentials: {e}");
            return Flow::Break(unauthorized_response("Invalid credentials", 401));
        }
    };

    // Validate authorization for the current user
    let validation = validator.authorize(&id);

    let client_data = match validation {
        Ok(client_data) => client_data,
        Err(e) => {
            logger::info!("Invalid authentication: {e}");
            return Flow::Break(unauthorized_response("Invalid authentication", 403));
        }
    };

    // Update the current authentication data
    if let Some(auth) = authentication.authentication() {
        authentication.set_authentication(Some(&AuthenticationData {
            client_id: Some(client_data.client_id),
            client_name: Some(client_data.client_name),
            ..auth
        }));
    }

    Flow::Continue(())
}

/// Polls the [ContractValidator] to keep the contracts database up to date.
async fn update_task(validator: &ContractValidator, clock: Clock) {
    let initialization_timer = clock.period(ContractValidator::INITIALIZATION_PERIOD);

    // Contracts database is polled at [ContractValidator::INITIALIZATION_PERIOD] rate.
    loop {
        if validator.update_contracts().await.is_ok() {
            logger::info!("Contracts storage initialized.");
            break;
        }

        if !initialization_timer.next_tick().await {
            logger::info!("Tick event suspended.");
            break;
        }

        logger::info!("Retrying contracts storage initialization.");
    }

    let update_timer = initialization_timer
        .release()
        .period(ContractValidator::UPDATE_PERIOD);

    // Contracts database is polled at [ContractValidator::UPDATE_PERIOD] rate.
    loop {
        let _ = validator.update_contracts().await;

        if !update_timer.next_tick().await {
            logger::info!("Tick event suspended.");
            break;
        }

        logger::info!("Retrying contracts storage initialization.");
    }
}

#[entrypoint]
async fn configure(
    launcher: Launcher,
    Configuration(ref conf): Configuration,
    clock: Clock,
    validator: ContractValidator,
) -> anyhow::Result<()> {
    let conf: Config = serde_json::from_slice(conf)?;

    let launch_task = async {
        match conf.mode.as_ref() {
            // Authorization mode
            "authorization" => {
                launcher
                    .launch(on_request(|rs, a| authorization_filter(rs, a, &validator)))
                    .await
            }
            _ => {
                // Authentication mode applied by default
                launcher
                    .launch(on_request(|rs, a| authentication_filter(rs, a, &validator)))
                    .await
            }
        }
    };

    // Run `update_task()` and `launch_task` concurrently.
    join! {
        update_task(&validator, clock),
        launch_task,
    }
    .1?;

    Ok(())
}
