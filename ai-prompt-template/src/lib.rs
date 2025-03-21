// Copyright 2023 Salesforce, Inc. All rights reserved.
mod applicator;
mod generated;
mod openai;

use anyhow::{anyhow, Result};

use applicator::TemplateApplicator;
use openai::Prompt;
use pdk::hl::*;
use pdk::logger;
use serde_json::json;

use crate::generated::config::Config;

/// Applies a template over a request
async fn apply_template(
    request_state: RequestState,
    applicator: &TemplateApplicator<'_>,
    allow_untemplated: bool,
) -> Result<(), (u32, &'static str)> {
    logger::info!("Applying template on incoming request.");

    let headers_state = request_state.into_headers_state().await;
    let headers_handler = headers_state.handler();

    // Removing content-length
    headers_handler.remove_header("content-length");

    let body_state = headers_state.into_body_state().await;

    // Skip requests without body
    if !body_state.contains_body() {
        logger::info!("Empty body.");
        return Ok(());
    }

    let handler = body_state.handler();
    let body = handler.body();

    // Deserialize prompt from incoming body.
    let prompt: Prompt =
        serde_json::from_slice(&body).map_err(|_| (400, "Unrecognized JSON structure"))?;

    let Some(template_name) = prompt.template_name() else {
        logger::info!("Prompt without template tag.");

        // Skip prompts without template tags.
        return Ok(());
    };

    // Try to apply the prompt properties on the selected template.
    let Some(application) = applicator.apply(template_name, prompt.properties) else {
        // Requested template not found.
        logger::info!("Template with name '{template_name}' not found.");

        return if allow_untemplated {
            logger::info!("Request skipped.");

            // Untemplated requests are allowed and skipped.
            Ok(())
        } else {
            logger::info!("Request refused.");

            // Untemplated requests are disallowed and refused.
            Err((400, "Template not found"))
        };
    };

    logger::info!("Template succesfully applied");

    handler
        .set_body(application.as_bytes())
        .map_err(|_| (400, "Payload too long."))?;

    Ok(())
}

async fn request_filter(
    request_state: RequestState,
    applicator: &TemplateApplicator<'_>,
    allow_untemplated: bool,
) -> Flow<()> {
    match apply_template(request_state, applicator, allow_untemplated).await {
        // Continue if it is ok
        Ok(_) => Flow::Continue(()),

        // Early response when error
        Err((status_code, error)) => Flow::Break(
            Response::new(status_code)
                .with_body(json!({"error": error}).to_string())
                .with_headers([("Content-Type".to_string(), "application/json".to_string())]),
        ),
    }
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let applicator = TemplateApplicator::from_config(&config);
    let filter =
        on_request(|rs| request_filter(rs, &applicator, config.allow_untemplated_requests));
    launcher.launch(filter).await?;

    Ok(())
}
