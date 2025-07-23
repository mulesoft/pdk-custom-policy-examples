// Copyright 2023 Salesforce, Inc. All rights reserved.

// Default configuration values
pub const DEFAULT_TTL_SECONDS: u32 = 60;
pub const DEFAULT_NAMESPACE: &str = "request-stats";
pub const DEFAULT_STORAGE_TYPE: &str = "local";

// HTTP header names
pub const CLIENT_ID_HEADER: &str = "x-client-id";
pub const REQUEST_COUNT_HEADER: &str = "x-request-count";
pub const LAST_REQUEST_HEADER: &str = "x-last-request";
pub const ALL_STATS_HEADER: &str = "x-all-stats";
pub const STATS_RESET_HEADER: &str = "x-stats-reset";

// Storage type values
pub const REMOTE_STORAGE: &str = "remote";
pub const LOCAL_STORAGE: &str = "local";
