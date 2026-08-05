// Copyright 2026 Salesforce, Inc. All rights reserved.
mod generated;

use std::collections::HashSet;

use anyhow::{anyhow, Result};
use pdk::hl::*;
use pdk::logger;
use regex::Regex;
use serde_json::Value;

use crate::generated::config::Config;

/// Classifies how a request body should be processed, derived from its
/// `content-type`. Each variant maps to a distinct redaction flow with its own
/// performance profile — this is what the criterion benchmark measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyKind {
    /// Body is left untouched (binary, unknown, or missing content type).
    Skip,
    /// Body is scanned as plain text with the configured regex patterns.
    Text,
    /// Body is parsed as JSON and walked recursively to mask fields and values.
    Json,
}

impl BodyKind {
    fn from_content_type(content_type: &str) -> Self {
        // Compare against the media type only, ignoring any `; charset=...` suffix.
        let media_type = content_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();

        match media_type.as_str() {
            "application/json" => BodyKind::Json,
            "text/plain" | "application/x-www-form-urlencoded" => BodyKind::Text,
            _ => BodyKind::Skip,
        }
    }
}

/// Holds the compiled redaction rules. Built once at policy configuration time
/// so the per-request filter never recompiles regular expressions.
pub struct Redactor {
    /// Header names to mask, stored lowercase for case-insensitive matching.
    sensitive_headers: Vec<String>,
    /// JSON object keys whose values are masked, at any nesting depth.
    mask_fields: HashSet<String>,
    /// All configured patterns combined into a single alternation, so a body is
    /// scanned in one pass instead of one pass per pattern. `None` when no
    /// patterns are configured.
    value_pattern: Option<Regex>,
    /// Token substituted for every redacted value.
    mask: String,
}

impl Redactor {
    /// Builds a [`Redactor`] from the policy [`Config`], compiling the combined
    /// value pattern. Returns an error if any configured pattern is invalid.
    pub fn from_config(config: &Config) -> Result<Self, regex::Error> {
        let sensitive_headers = config
            .sensitive_headers
            .iter()
            .map(|h| h.to_ascii_lowercase())
            .collect();

        let mask_fields = config.mask_fields.iter().cloned().collect();

        let value_pattern = if config.patterns.is_empty() {
            None
        } else {
            // Wrap each pattern in a non-capturing group before joining so
            // top-level alternation in a single pattern cannot leak across.
            let combined = config
                .patterns
                .iter()
                .map(|p| format!("(?:{p})"))
                .collect::<Vec<_>>()
                .join("|");
            Some(Regex::new(&combined)?)
        };

        Ok(Self {
            sensitive_headers,
            mask_fields,
            value_pattern,
            mask: config.mask_with.clone(),
        })
    }

    /// Reports whether this redactor could modify a body of the given kind.
    /// When it cannot, the filter skips reading the body altogether — awaiting
    /// the body state forces the runtime to buffer the payload, so avoiding it
    /// keeps a no-op flow cheap.
    pub fn touches_body(&self, kind: BodyKind) -> bool {
        match kind {
            // Text is only ever rewritten by the value patterns.
            BodyKind::Text => self.value_pattern.is_some(),
            // JSON can be rewritten by either field masking or value patterns.
            BodyKind::Json => self.value_pattern.is_some() || !self.mask_fields.is_empty(),
            BodyKind::Skip => false,
        }
    }

    /// Replaces every pattern match in `input` with the mask token. Returns
    /// `None` when no patterns are configured, so the caller can skip rewriting
    /// the body entirely.
    pub fn redact_text(&self, input: &str) -> Option<String> {
        let pattern = self.value_pattern.as_ref()?;
        Some(pattern.replace_all(input, self.mask.as_str()).into_owned())
    }

    /// Recursively masks a JSON value in place: any object key listed in
    /// `mask_fields` has its value fully replaced, and every remaining string is
    /// scanned with the configured patterns.
    pub fn redact_json(&self, value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, child) in map.iter_mut() {
                    if self.mask_fields.contains(key) {
                        *child = Value::String(self.mask.clone());
                    } else {
                        self.redact_json(child);
                    }
                }
            }
            Value::Array(items) => {
                for item in items.iter_mut() {
                    self.redact_json(item);
                }
            }
            Value::String(text) => {
                if let Some(pattern) = self.value_pattern.as_ref() {
                    if pattern.is_match(text) {
                        *text = pattern.replace_all(text, self.mask.as_str()).into_owned();
                    }
                }
            }
            _ => {}
        }
    }
}

/// Redacts a single request. The flow is chosen from the request itself — the
/// sensitive headers it carries and its `content-type` — never from an
/// out-of-band selector, so the emulated benchmark exercises the same decision
/// path production traffic would.
async fn request_filter(request_state: RequestState, redactor: &Redactor) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;
    let handler = headers_state.handler();

    // Header flow: mask any configured sensitive header that is present. Cheap
    // and always runs, so it happens before we decide what to do with the body.
    for name in &redactor.sensitive_headers {
        if handler.header(name).is_some() {
            handler.set_header(name, &redactor.mask);
        }
    }

    // Body flow selection: content-type decides which redaction path (if any)
    // the body takes.
    let content_type = handler.header("content-type").unwrap_or_default();
    let kind = BodyKind::from_content_type(&content_type);

    // Skip the body entirely when this flow could never modify it (unredactable
    // content type, or no rules that apply to it). Awaiting the body state forces
    // the runtime to buffer the payload, so this keeps no-op flows cheap.
    if !redactor.touches_body(kind) {
        return Flow::Continue(());
    }

    // The masked body may differ in length; drop the stale content-length now,
    // while the headers handler is still available, so the runtime recomputes it
    // from the payload we forward.
    handler.remove_header("content-length");

    let body_state = headers_state.into_body_state().await;
    if !body_state.contains_body() {
        return Flow::Continue(());
    }

    let body_handler = body_state.handler();
    let body = body_handler.body();

    let rewritten = match kind {
        BodyKind::Text => std::str::from_utf8(&body)
            .ok()
            .and_then(|text| redactor.redact_text(text))
            .map(String::into_bytes),
        BodyKind::Json => serde_json::from_slice::<Value>(&body)
            .ok()
            .map(|mut value| {
                redactor.redact_json(&mut value);
                serde_json::to_vec(&value).unwrap_or(body.clone())
            }),
        BodyKind::Skip => None,
    };

    if let Some(bytes) = rewritten {
        if let Err(err) = body_handler.set_body(&bytes) {
            logger::error!("Unable to set redacted body: {err:?}");
        }
    }

    Flow::Continue(())
}

#[entrypoint]
pub async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {err}",
            String::from_utf8_lossy(&bytes),
        )
    })?;

    let redactor = Redactor::from_config(&config)
        .map_err(|err| anyhow!("Invalid redaction pattern. Cause: {err}"))?;

    let filter = on_request(|rs| request_filter(rs, &redactor));
    launcher.launch(filter).await?;
    Ok(())
}

#[cfg(test)]
mod tests;
