// Copyright 2023 Salesforce, Inc. All rights reserved.
use anyhow::Result;
use pdk_test::services::httpbin::HttpBinConfig;
use serde::Serialize;

use pdk_test::port::Port;
use pdk_test::TestComposite;

use pdk_test::services::flex::{ApiConfig, FlexConfig, PolicyConfig};

// This module contains common Rust stuff shared between test files.

// Directory where the policies implementations are stored.
pub const POLICY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/target/wasm32-wasip1/release");

// Directory with the common configurations for tests.
pub const COMMON_CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config");

// In case the project name changes, override this value with the actual policy name.
// To obtain the current name, run the "make show-policy-ref-name" goal, or read it from
// "target/policy-ref-name.txt" after building the project.
pub const POLICY_NAME: &str = "cors-validation-v1-0-impl";

// Default Flex port.
pub const FLEX_PORT: Port = 8081;

pub async fn compose<T: Serialize>(config: &T) -> Result<TestComposite> {
    let httpbin_config = HttpBinConfig::builder().hostname("httpbin").build();

    let policy_config = PolicyConfig::builder()
        .name(POLICY_NAME)
        .configuration(config)
        .build();

    let api = ApiConfig::builder()
        .name("myApi")
        .upstream(&httpbin_config)
        .path("/anything/echo/")
        .port(FLEX_PORT)
        .policies([policy_config])
        .build();

    let flex_config = FlexConfig::builder()
        .version("1.10.0")
        .with_api(api)
        .config_mounts([
            (COMMON_CONFIG_DIR, "common"),
            (POLICY_DIR, "implementation"),
        ])
        .build();

    let composite = TestComposite::builder()
        .with_service(httpbin_config)
        .with_service(flex_config)
        .build()
        .await?;

    Ok(composite)
}
