// Copyright 2023 Salesforce, Inc. All rights reserved.
use anyhow::Result;
use serde_json::json;

use pdk_test::pdk_test;
use pdk_test::services::flex::Flex;

mod common;
use common::{compose, FLEX_PORT};
use reqwest::{Method, StatusCode};

const ACCESS_CONTROL_ALLOW_METHODS_HEADER: &str = "access-control-allow-methods";
const ACCESS_CONTROL_ALLOW_ORIGIN_KEY: &str = "Access-Control-Allow-Origin";
const ACCESS_CONTROL_ALLOW_ORIGIN_VALUE: &str = "http://localhost:8081";

#[pdk_test]
async fn correct_headers() -> Result<()> {

    // Policy configuration
    let config = json!({
        "originGroups": {
            "accessControlMaxAge": 30,
            "origins": [ "http://www.the-origin-of-time.com" ],
            "allowedMethods": [
                {
                    "methodName": "GET",
                    "allowed": true
                },
                {
                    "methodName": "PUSH",
                    "allowed": true
                }
            ],
            "headers": [
                "content-type",
                "x-allow-origin",
                "x-yet-another-valid-header"
            ],
            "exposedHeaders": [ "x-forwarded-for" ]
        }
    });

    let composite = compose(&config).await?;

    let flex: Flex = composite.service()?;

    let client = reqwest::Client::new();

    let response = client
        .request(Method::OPTIONS, flex.external_url(FLEX_PORT).unwrap())
        .header("origin", "http://www.the-origin-of-time.com")
        .header("access-control-request-method", "GET")
        .send()
        .await?;

    let response_headers = response.headers();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response_headers[ACCESS_CONTROL_ALLOW_METHODS_HEADER].to_str()?,
        "GET"
    );

    Ok(())
}

// Duplicated headers are allowed.
#[pdk_test]
async fn header_duplication() -> Result<()> {

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
