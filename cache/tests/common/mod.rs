// Copyright 2023 Salesforce, Inc. All rights reserved.

// This module contains common Rust stuff shared between test files.

// Directory where the policies implementations are stored.
pub const POLICY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/target/wasm32-wasip1/release");

// Directory with the common configurations for tests.
pub const COMMON_CONFIG_DIR: &str =  concat!(env!("CARGO_MANIFEST_DIR"), "/tests/config");

// In case the project name changes, override this value with the actual policy name.
// To obtain the current name, run the "make show-policy-ref-name" goal, or read it from
// "target/policy-ref-name.txt" after building the project.
pub const POLICY_NAME: &str = "cache-flex-v1-0";
