// Copyright 2026 Salesforce, Inc. All rights reserved.

//! Unit tests for the PII redaction policy, driven through the in-process
//! `pdk-unit` emulated environment. Each test crafts a request that routes to a
//! specific flow and asserts on what the backend actually received.

mod fixtures;

use std::rc::Rc;

use pdk_unit::{TraceBackend, UnitHttpMessage, UnitHttpRequest, UnitHttpResponse, UnitTestBuilder};

/// A capturing backend built over a plain `fn` response producer.
type CaptureBackend = TraceBackend<fn(UnitHttpRequest) -> UnitHttpResponse>;

/// Backend that echoes a 200 and records every request it receives, so tests can
/// assert on the (redacted) request the policy forwarded upstream.
fn ok_backend(_req: UnitHttpRequest) -> UnitHttpResponse {
    UnitHttpResponse::new(200)
}

/// Builds a tester wired with a capturing backend and the standard config.
/// Returns both the tester and the trace handle for inspecting forwarded requests.
fn tester() -> (pdk_unit::UnitTest, Rc<CaptureBackend>) {
    let backend: Rc<CaptureBackend> = Rc::new(TraceBackend::new(ok_backend));
    let tester = UnitTestBuilder::default()
        .with_config(fixtures::config())
        .with_backend(Rc::clone(&backend))
        .with_entrypoint(crate::configure);
    (tester, backend)
}

#[test]
fn passthrough_leaves_binary_body_untouched() {
    let (mut tester, backend) = tester();

    let body = vec![0u8, 1, 2, 3, 255, 254];
    let response = tester.request(
        UnitHttpRequest::post()
            .with_header("content-type", "image/png")
            .with_body(body.clone()),
    );
    assert_eq!(response.status_code(), 200);

    let forwarded = backend.next().expect("backend received request");
    assert_eq!(forwarded.body(), body.as_slice());
}

#[test]
fn header_redact_masks_sensitive_headers() {
    let (mut tester, backend) = tester();

    let response = tester.request(
        UnitHttpRequest::get()
            .with_header("authorization", "Bearer super-secret-token")
            .with_header("x-api-key", "abcdef123456")
            .with_header("x-request-id", "keep-me"),
    );
    assert_eq!(response.status_code(), 200);

    let forwarded = backend.next().expect("backend received request");
    assert_eq!(forwarded.header("authorization"), Some("***REDACTED***"));
    assert_eq!(forwarded.header("x-api-key"), Some("***REDACTED***"));
    // Non-sensitive headers pass through unchanged.
    assert_eq!(forwarded.header("x-request-id"), Some("keep-me"));
}

#[test]
fn regex_scan_masks_values_in_text_body() {
    let (mut tester, backend) = tester();

    let response = tester.request(
        UnitHttpRequest::post()
            .with_header("content-type", "text/plain")
            .with_body(fixtures::text_body()),
    );
    assert_eq!(response.status_code(), 200);

    let forwarded = backend.next().expect("backend received request");
    let body = String::from_utf8(forwarded.body().to_vec()).unwrap();
    assert!(!body.contains("jane.doe@example.com"), "email must be masked");
    assert!(!body.contains("123-45-6789"), "ssn must be masked");
    assert!(!body.contains("4111 1111 1111 1111"), "card must be masked");
    assert!(body.contains("***REDACTED***"));
    // Ordinary content survives.
    assert!(body.contains("ordinary"));
}

#[test]
fn json_mask_redacts_shallow_fields() {
    let (mut tester, backend) = tester();

    let response = tester.request(
        UnitHttpRequest::post()
            .with_header("content-type", "application/json")
            .with_body(fixtures::json_shallow_body()),
    );
    assert_eq!(response.status_code(), 200);

    let forwarded = backend.next().expect("backend received request");
    let value: serde_json::Value = serde_json::from_slice(forwarded.body()).unwrap();
    assert_eq!(value["email"], "***REDACTED***");
    assert_eq!(value["ssn"], "***REDACTED***");
    assert_eq!(value["cardNumber"], "***REDACTED***");
    assert_eq!(value["password"], "***REDACTED***");
    // Non-sensitive field survives.
    assert_eq!(value["name"], "Jane Doe");
    // A pattern match inside a non-masked field is still redacted by regex.
    assert_eq!(value["note"], "reach me at ***REDACTED***");
}

#[test]
fn json_deep_redacts_nested_fields() {
    let (mut tester, backend) = tester();

    let response = tester.request(
        UnitHttpRequest::post()
            .with_header("content-type", "application/json")
            .with_body(fixtures::json_deep_body()),
    );
    assert_eq!(response.status_code(), 200);

    let forwarded = backend.next().expect("backend received request");
    let body = String::from_utf8(forwarded.body().to_vec()).unwrap();
    assert!(!body.contains("jane.doe@example.com"), "nested email masked");
    assert!(!body.contains("123-45-6789"), "nested ssn masked");
    assert!(!body.contains("4111 1111 1111 1111"), "nested card masked");
    assert!(!body.contains("hunter2"), "nested password masked");
    // Structural / non-sensitive data survives the deep walk.
    assert!(body.contains("us-east"));
}
