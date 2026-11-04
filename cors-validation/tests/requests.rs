// Copyright 2023 Salesforce, Inc. All rights reserved.
use anyhow::Result;
use serde_json::json;

use pdk_test::pdk_test;
use pdk_test::services::flex::Flex;

mod common;
use common::{compose, FLEX_PORT};
use reqwest::StatusCode;

const ACCESS_CONTROL_ALLOW_ORIGIN_KEY: &str = "Access-Control-Allow-Origin";
const ACCESS_CONTROL_ALLOW_ORIGIN_VALUE: &str = "http://localhost:8081";

#[pdk_test]
async fn check_origin_header() -> Result<()> {
    // Policy configuration
    let config = json!( {
        "publicResource": false,
        "supportCredentials": false,
        "originGroups": [{
            "accessControlMaxAge": 30,
            "origins": [ ACCESS_CONTROL_ALLOW_ORIGIN_VALUE ],
            "allowedMethods": [{
                "methodName": "GET",
                "allowed": true
            }]
        }]
    });

    let composite = compose(&config).await?;
    let flex: Flex = composite.service()?;

    let client = reqwest::Client::new();

    let response = client
        .get(flex.external_url(FLEX_PORT).unwrap())
        .header("Origin", "http://localhost:8081")
        .send()
        .await?;

    let headers = response.headers();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        headers[ACCESS_CONTROL_ALLOW_ORIGIN_KEY].to_str()?,
        ACCESS_CONTROL_ALLOW_ORIGIN_VALUE
    );

    Ok(())
}
