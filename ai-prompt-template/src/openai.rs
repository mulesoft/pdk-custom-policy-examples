use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents an OpenAI prompt request.
#[derive(Serialize, Deserialize, Debug)]
pub struct Prompt<'a> {
    pub prompt: &'a str,
    pub properties: HashMap<&'a str, &'a str>,
}

impl<'a> Prompt<'a> {
    /// Returns the associated template name if exists.
    pub fn template_name(&self) -> Option<&'a str> {
        self.prompt
            .strip_prefix("{template://")
            .and_then(|s| s.strip_suffix("}"))
    }
}
