// Copyright 2023 Salesforce, Inc. All rights reserved.

// This module contains common Rust stuff shared between test files.

// Directory where the policies implementations are stored.
pub const POLICY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/target/wasm32-wasi/release");

// Directory with the common configurations for tests.
pub const COMMON_CONFIG_DIR: &str =  concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common");
