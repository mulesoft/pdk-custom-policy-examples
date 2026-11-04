// Copyright 2026 Salesforce, Inc. All rights reserved.

//! PII redaction benchmarks driven through the pdk-unit emulated environment.
//!
//! Each group issues one HTTP request per iteration. The request's content type
//! and headers alone decide which redaction flow the policy runs — there is no
//! out-of-band selector — so the groups measure the flows a real deployment
//! would take. The measured time includes the pdk-unit request/host emulation
//! overhead in addition to the redaction work itself; treat the numbers as
//! relative (flow vs flow) rather than as isolated redaction latencies.
//!
//! Flows, cheapest to most expensive:
//!   * `passthrough`   — binary content type, body left untouched
//!   * `header_redact` — mask sensitive headers, no body work
//!   * `regex_scan`    — regex sweep over a plain-text body
//!   * `json_mask`     — parse + walk a shallow JSON object
//!   * `json_deep`     — recursive walk over a deeply nested JSON document

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pdk_unit::{UnitHttpRequest, UnitLogLevel, UnitTest, UnitTestBuilder};

/// Config and sample bodies driving each flow. A benchmark is a separate crate
/// that cannot import the policy's `#[cfg(test)]` fixtures, so these are
/// duplicated from `src/tests/fixtures.rs` — keep the two copies in sync.
mod fixtures {
    use serde_json::json;

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

    pub fn text_body() -> String {
        let block = "Contact jane.doe@example.com or call about SSN 123-45-6789. \
                     Card on file 4111 1111 1111 1111. Everything else is ordinary \
                     log line content that must survive untouched.\n";
        block.repeat(16)
    }

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
}

/// Builds a tester with the standard redaction config and a default 200 backend.
fn tester() -> UnitTest {
    let mut tester = UnitTestBuilder::default()
        .with_config(fixtures::config())
        .with_entrypoint(pii_redaction::configure);
    tester.set_log_level(UnitLogLevel::Disabled);
    tester
}

fn bench_redaction_flows(c: &mut Criterion) {
    let mut group = c.benchmark_group("pii_redaction");

    // Passthrough: a binary content type the policy does not touch.
    let mut t = tester();
    group.bench_function("passthrough", |b| {
        let body = vec![0u8, 1, 2, 3, 255, 254, 128, 64];
        b.iter(|| {
            black_box(
                t.request(
                    UnitHttpRequest::post()
                        .with_header("content-type", "image/png")
                        .with_body(body.clone()),
                ),
            );
        });
    });

    // Header redaction: mask sensitive headers, no body processing.
    let mut t = tester();
    group.bench_function("header_redact", |b| {
        b.iter(|| {
            black_box(
                t.request(
                    UnitHttpRequest::get()
                        .with_header("authorization", "Bearer super-secret-token")
                        .with_header("cookie", "session=abcdef123456")
                        .with_header("x-api-key", "key-0123456789")
                        .with_header("x-request-id", "keep-me"),
                ),
            );
        });
    });

    // Regex scan: sweep a plain-text body for sensitive value patterns.
    let mut t = tester();
    group.bench_function("regex_scan", |b| {
        let body = fixtures::text_body();
        b.iter(|| {
            black_box(
                t.request(
                    UnitHttpRequest::post()
                        .with_header("content-type", "text/plain")
                        .with_body(body.clone()),
                ),
            );
        });
    });

    // JSON mask: parse and walk a shallow JSON object.
    let mut t = tester();
    group.bench_function("json_mask", |b| {
        let body = fixtures::json_shallow_body();
        b.iter(|| {
            black_box(
                t.request(
                    UnitHttpRequest::post()
                        .with_header("content-type", "application/json")
                        .with_body(body.clone()),
                ),
            );
        });
    });

    // JSON deep: recursive walk over a deeply nested JSON document.
    let mut t = tester();
    group.bench_function("json_deep", |b| {
        let body = fixtures::json_deep_body();
        b.iter(|| {
            black_box(
                t.request(
                    UnitHttpRequest::post()
                        .with_header("content-type", "application/json")
                        .with_body(body.clone()),
                ),
            );
        });
    });

    group.finish();
}

criterion_group!(benches, bench_redaction_flows);
criterion_main!(benches);
