// Copyright 2023 Salesforce, Inc. All rights reserved.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents an OpenAI API chat message.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Message<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

/// Represents an OpenAI API chat completion.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Completion<'a> {
    pub model: &'a str,
    pub messages: Vec<Message<'a>>,

    #[serde(flatten)]
    pub extra: HashMap<&'a str, Value>,
}
