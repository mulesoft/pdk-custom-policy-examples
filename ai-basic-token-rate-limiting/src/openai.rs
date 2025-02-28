// Copyright 2023 Salesforce, Inc. All rights reserved.
use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct Message<'a> {
    #[allow(unused)]
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct Completion<'a> {
    pub messages: Vec<Message<'a>>,

    #[allow(unused)]
    pub model: &'a str,

    #[allow(unused)]
    #[serde(flatten)]
    pub extra: HashMap<&'a str, Value>,
}
