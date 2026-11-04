// Copyright 2026 Salesforce, Inc. All rights reserved.

//! Test fixtures: the policy configuration and the sample request bodies that
//! drive each redaction flow. These are test-only (this module is compiled under
//! `#[cfg(test)]`) and are intentionally duplicated in `benches/redaction.rs`,
//! since a benchmark is a separate crate that cannot import test code. Keep the
//! two copies in sync so a bench number corresponds to verified behavior.

use serde_json::json;

/// Standard policy configuration: masks common auth headers, a handful of PII
/// JSON fields, and value patterns for emails, US SSNs, and credit card numbers.
pub fn config() -> String {
    json!({
        "sensitiveHeaders": ["authorization", "cookie", "x-api-key"],
        "maskFields": ["ssn", "email", "cardNumber", "password"],
        "patterns": [
            // Email
            r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}",
            // US Social Security Number
            r"\b\d{3}-\d{2}-\d{4}\b",
            // 16-digit credit card number (optionally grouped)
            r"\b(?:\d[ -]?){15}\d\b"
        ],
        "maskWith": "***REDACTED***"
    })
    .to_string()
}

/// A plain-text body containing several sensitive values, used by the
/// `regex-scan` flow. Repeated so the scan does non-trivial work.
pub fn text_body() -> String {
    let block = "Contact jane.doe@example.com or call about SSN 123-45-6789. \
                 Card on file 4111 1111 1111 1111. Everything else is ordinary \
                 log line content that must survive untouched.\n";
    block.repeat(16)
}

/// A shallow JSON body: sensitive data lives in top-level fields, used by the
/// `json-mask` flow.
pub fn json_shallow_body() -> String {
    json!({
        "name": "Jane Doe",
        "email": "jane.doe@example.com",
        "ssn": "123-45-6789",
        "cardNumber": "4111 1111 1111 1111",
        "password": "hunter2",
        "note": "reach me at jane.doe@example.com"
    })
    .to_string()
}

/// A deeply nested JSON body: sensitive data is buried under several object and
/// array levels, used by the `json-deep` flow. The recursive walk must descend
/// the whole tree, so this is the most expensive flow.
pub fn json_deep_body() -> String {
    json!({
        "account": {
            "owner": {
                "email": "jane.doe@example.com",
                "ssn": "123-45-6789",
                "profile": {
                    "password": "hunter2",
                    "history": [
                        {"cardNumber": "4111 1111 1111 1111", "amount": 12},
                        {"cardNumber": "5500 0000 0000 0004", "amount": 34},
                        {"note": "billed jane.doe@example.com", "amount": 56}
                    ]
                }
            },
            "metadata": {
                "region": "us-east",
                "contacts": [
                    {"email": "ops@example.com"},
                    {"email": "billing@example.com"}
                ]
            }
        }
    })
    .to_string()
}
