// Copyright 2023 Salesforce, Inc. All rights reserved.

// Default configuration values
pub const DEFAULT_TTL_SECONDS: u32 = 60;
pub const DEFAULT_NAMESPACE: &str = "request-stats";
pub const DEFAULT_STORAGE_TYPE: &str = "local";

// HTTP header for client identification (input only)
pub const CLIENT_ID_HEADER: &str = "x-client-id";

// Storage type values
pub const REMOTE_STORAGE: &str = "remote";
pub const LOCAL_STORAGE: &str = "local";
