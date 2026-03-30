// Copyright 2023 Salesforce, Inc. All rights reserved.
use anyhow::{anyhow, Result};
use chrono::Utc;
use pdk::hl::*;
use pdk::jwt::*;
use pdk::logger::debug;

use crate::generated::config::Config;

mod generated;

async fn filter(
    state: RequestState,
    config: &Config,
    signature_validator: &SignatureValidator,
) -> Flow<()> {
    let headers_state = state.into_headers_state().await;

    // Extract token
    let token = match TokenProvider::bearer(headers_state.handler()) {
        Ok(t) => t,
        Err(_) => {
            return Flow::Break(Response::new(401).with_body("Bearer not found"));
        }
    };

    // Validate signature
    let claims = match signature_validator.validate(token) {
        Ok(claims) => claims,
        Err(_) => {
            return Flow::Break(Response::new(401).with_body("Invalid token"));
        }
    };

    // Validate expiration
    if let Some(exp) = claims.expiration() {
        if exp < Utc::now() {
            return Flow::Break(Response::new(401).with_body("Expired token"));
        }
    } else {
        return Flow::Break(Response::new(401).with_body("Token missing exp claim"));
    }

    // Custom claim validation
    let mut evaluator = config.custom_rule.evaluator();
    evaluator.bind_vars("claimSet", claims.get_claims());

    if !evaluator
        .eval()
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or_default()
    {
        return Flow::Break(
            Response::new(400).with_body("Invalid token: Only members are allowed."),
        );
    }

    // Access JWT payload on demand
    if let Some(not_before) = claims.not_before() {
        debug!("not_before is {}'", not_before);
    }

    // Access JWT headers on demand
    if let Some(alg_header) = claims.get_header("alg") {
        debug!("alg header is {}", alg_header);
    }

    // Propagate claims to headers
    let some_custom_claim: Option<String> = claims.get_claim("username");

    if let Some(custom_claim) = some_custom_claim {
        headers_state
            .handler()
            .set_header("username", custom_claim.as_str());
    }

    Flow::Continue(())
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(configuration): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&configuration).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&configuration),
            err
        )
    })?;

    let signature_validator = SignatureValidator::new(
        model::SigningAlgorithm::Hmac,
        model::SigningKeyLength::Len256,
        config.secret.clone(),
    )?;

    launcher
        .launch(on_request(|request| {
            filter(request, &config, &signature_validator)
        }))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{dw2pel, UnitHttpRequest, UnitTestBuilder};
    use serde_json::json;

    const SECRET: &str = "Make-sure-this-is-a-secret-dont-share-it";

    const VALID_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImNsYXNzaWQiOjQzOX0.eyJpc3MiOiJMaWJyYXJ5Iiwic3ViIjoiMTIzNDUiLCJhdWQiOiJtZW1iZXItZ3JvdXAiLCJpYXQiOjE3MDQ0NjA0MDcsIm5iZiI6MTcwNDQ2MDQwNywiZXhwIjoyNzA0NDYwNDA3LCJ1c2VybmFtZSI6IkxpYnJhcnlGYW4xOTg0Iiwicm9sZSI6Ik1lbWJlciJ9.-100JFDt5ET4DA0hFnCRQKk5BNok0LCCF6jqyNU19sE";

    const EXPIRED_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImNsYXNzaWQiOjQzOX0.eyJpc3MiOiJMaWJyYXJ5Iiwic3ViIjoiMTIzNDUiLCJhdWQiOiJtZW1iZXItZ3JvdXAiLCJpYXQiOjE3MDQ0NjA0MDcsIm5iZiI6MTcwNDQ2MDQwNywiZXhwIjoxNzA0NDYxNDA3LCJ1c2VybmFtZSI6IkxpYnJhcnlGYW4xOTg0Iiwicm9sZSI6Ik1lbWJlciJ9.51yQLhxGV9IYK8XYF8rSIwne5ZrgxxeQgkCcHidOuZE";

    fn config() -> String {
        json!({
            "secret": SECRET,
            "customRule": dw2pel("vars.claimSet.role == 'Member'"),
        })
        .to_string()
    }

    #[test]
    fn valid_token_passes_through() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::get().with_header("Authorization", format!("Bearer {VALID_TOKEN}")),
        );

        assert_eq!(response.status_code(), 200);
    }

    #[test]
    fn missing_bearer_returns_401() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 401);
    }

    #[test]
    fn expired_token_returns_401() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::get().with_header("Authorization", format!("Bearer {EXPIRED_TOKEN}")),
        );

        assert_eq!(response.status_code(), 401);
    }

    #[test]
    fn invalid_signature_returns_401() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request_full(
            UnitHttpRequest::get().with_header("Authorization", "Bearer invalid.jwt.token"),
        );

        assert_eq!(response.status_code(), 401);
    }
}
