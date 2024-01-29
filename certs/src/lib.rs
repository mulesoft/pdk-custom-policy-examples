// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;

const EMAIL_SUBJECT_PREAMBLE: &str = "emailAddress=";
const NAME_SUBJECT_PREAMBLE: &str = "CN=";

/// This function reads the property "path" from the StreamProperties and returns is as a String.
fn read_property(stream: &StreamProperties, path: &[&str]) -> String {
    let bytes = stream.read_property(path).unwrap_or_default();
    String::from_utf8_lossy(&bytes).to_string()
}

/// Struct that contains the data we are interested in extracted from the subject field.
pub struct Subject {
    name: String,
    email: String,
}

/// This function extracts the name and email from the given subject field.
fn parse_subject(subject_field: &str) -> Result<Subject> {
    let split = subject_field.split(',');
    let mut email = None;
    let mut name = None;
    for segment in split {
        // We extract the email.
        if segment.starts_with(EMAIL_SUBJECT_PREAMBLE) {
            email = Some(segment.split_at(EMAIL_SUBJECT_PREAMBLE.len()).1.to_string())
        }
        // We extract the name.
        else if segment.starts_with(NAME_SUBJECT_PREAMBLE) {
            name = Some(segment.split_at(NAME_SUBJECT_PREAMBLE.len()).1.to_string())
        }
    }

    Ok(Subject {
        name: name.ok_or(anyhow!("Common name missing from peer cert."))?,
        email: email.ok_or(anyhow!("Email missing from peer cert."))?,
    })
}

/// This filter reads the subject field from the peer certificate and adds the name and email as headers.
async fn request_filter(request_state: RequestState, stream: StreamProperties) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;

    match parse_subject(
        // For more data from the connection certificates check: https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/attributes#connection-attributes.
        // For upstream certificate data check: https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/advanced/attributes#upstream-attribute.
        read_property(&stream, &["connection", "subject_peer_certificate"]).as_str(),
    ) {
        Err(err) => Flow::Break(Response::new(401).with_body(err.to_string())),
        Ok(resp) => {
            headers_state
                .handler()
                .set_header("X-Peer-Name", resp.name.as_str());
            headers_state
                .handler()
                .set_header("X-Peer-Email", resp.email.as_str());
            Flow::Continue(())
        }
    }
}

#[entrypoint]
async fn configure(launcher: Launcher) -> Result<()> {
    let filter = on_request(request_filter);
    launcher.launch(filter).await?;
    Ok(())
}
