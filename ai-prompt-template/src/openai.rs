// Copyright 2023 Salesforce, Inc. All rights reserved.
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Prompt;

    #[test]
    fn extract_template_name() {
        let prompt = Prompt {
            prompt: "{template://my-template}",
            properties: HashMap::default(),
        };

        assert_eq!(prompt.template_name(), Some("my-template"));
    }

    #[test]
    fn missing_template_name() {
        let prompt = Prompt {
            prompt: "{foo://bar}",
            properties: HashMap::default(),
        };

        assert_eq!(prompt.template_name(), None);
    }
}
