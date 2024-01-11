// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;
use crate::generated::config::Config;
use anyhow::Result;
use chrono::Utc;
use pdk::hl::*;
use pdk::jwt::model::ScriptClaimRule;
use pdk::jwt::*;
use pdk::logger::debug;
use pdk::script::HandlerAttributesBinding;

async fn filter(
    state: RequestState,
    signature_validator: &SignatureValidator,
    custom_validator: &ScriptClaimValidator,
) -> Flow<()> {
    let headers_state = state.into_headers_state().await;

    // Extract token
    let token = TokenProvider::BearerProvider
        .provide(&HandlerAttributesBinding::partial(headers_state.handler()));

    if token.is_err() {
        return Flow::Break(Response::new(401).with_body("Bearer not found"));
    }

    // Validating signature
    let claims = signature_validator.validate(token.unwrap());

    if claims.is_err() {
        return Flow::Break(Response::new(401).with_body("Invalid token"));
    }

    // Validating token expiration
    let claims = claims.unwrap();
    let exp_validator = ClaimValidator::Exp {
        mandatory: true,
        current_time: Utc::now(),
    };

    let exp_validation = exp_validator.validate(&claims);

    if exp_validation.is_err() {
        return Flow::Break(Response::new(401).with_body("Expired token"));
    }

    // Custom claim validation
    let custom_validation = custom_validator.validate(
        &claims,
        &HandlerAttributesBinding::partial(headers_state.handler()),
    );

    if custom_validation.is_err() {
        return Flow::Break(
            Response::new(400).with_body("Invalid token: Only authenticated customers allowed"),
        );
    }

    // Accessing JWT payload on demand
    let not_before = claims.not_before();

    if not_before.is_some() {
        debug!("not_before is {}'", not_before.unwrap());
    }

    // Accessing JWT headers on demand
    let alg_header = claims.get_header("alg");

    if alg_header.is_some() {
        debug!("alg header is {}", alg_header.unwrap());
    }

    // Propagating claims to headers
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
    let config: Config = serde_json::from_slice(&configuration)?;

    let signature_validator = SignatureValidator::new(
        model::SigningAlgorithm::Hmac,
        model::SigningKeyLength::Len256,
        config.clone().secret,
    )
    .unwrap();

    let custom_rules = [ScriptClaimRule::new(
        String::from("role"),
        &config.custom_rule,
    )]
    .to_vec();

    let custom_validator: ScriptClaimValidator =
        ScriptClaimValidator::new(custom_rules, Vec::new());

    launcher
        .launch(on_request(|request| {
            filter(request, &signature_validator, &custom_validator)
        }))
        .await?;
    Ok(())
}
