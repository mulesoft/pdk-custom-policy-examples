// Copyright 2023 Salesforce, Inc. All rights reserved.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents an OpenAI chat completion.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Completion<'a> {
    pub model: &'a str,
    pub messages: Vec<Message<'a>>,

    #[serde(flatten)]
    pub extra: HashMap<&'a str, Value>,
}

/// Represents an OpenAI chat message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message<'a> {
    pub role: &'a str,
    pub content: &'a str,
}
