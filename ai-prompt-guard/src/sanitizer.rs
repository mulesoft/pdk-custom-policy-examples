// Copyright 2023 Salesforce, Inc. All rights reserved.
use regex::RegexSet;

use crate::{generated::config::Config, openai::Completion};

/// Sanitizes [Completion]s by applying [Config] filters.
pub struct CompletionSanitizer {
    block: RegexSet,
    omit: RegexSet,
}

impl CompletionSanitizer {
    /// Creates a new [CompletionSanitizer] from a [Config].
    pub fn from_config(config: Config) -> Result<Self, regex::Error> {
        let (omit, block): (Vec<_>, Vec<_>) = config
            .filters
            .into_iter()
            .partition(|f| f.omit_instead_of_blocking);

        Ok(Self {
            block: RegexSet::new(block.into_iter().map(|f| f.pattern))?,
            omit: RegexSet::new(omit.into_iter().map(|f| f.pattern))?,
        })
    }

    /// Sanitizes a [Completion] by renoving messages to omit.
    /// Returns [None] if a block match is found.
    pub fn sanitize<'a>(&self, completion: Completion<'a>) -> Option<Completion<'a>> {
        let messages = completion
            .messages
            .into_iter()
            .map(|m| (!self.block.is_match(m.content)).then_some(m))
            .filter(|m| !m.as_ref().is_some_and(|m| self.omit.is_match(m.content)))
            .collect::<Option<_>>()?;

        Some(Completion {
            messages,
            ..completion
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        generated::config::{Config, Filters0Config as Filter},
        openai::{Completion, Message},
    };

    use super::CompletionSanitizer;

    fn make_sanitizer() -> CompletionSanitizer {
        CompletionSanitizer::from_config(Config {
            filters: vec![
                Filter {
                    // email
                    pattern: r#"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"#.to_string(),
                    omit_instead_of_blocking: true,
                },
                Filter {
                    // phone number
                    pattern: r#"(\+?\d{1,3})?[-.\s]?\(?\d{2,4}\)?[-.\s]?\d{3,4}[-.\s]?\d{4}"#
                        .to_string(),
                    omit_instead_of_blocking: false,
                },
            ],
        })
        .unwrap()
    }

    #[test]
    fn omit() {
        let sanitizer = make_sanitizer();

        let completion = Completion {
            model: "llama",
            messages: vec![
                // Remove
                Message {
                    role: "user",
                    content: "Their email is pdk@flex.com",
                },
                // Keep
                Message {
                    role: "user",
                    content: "Their name is PDK",
                },
            ],
            extra: HashMap::default(),
        };

        let actual = sanitizer.sanitize(completion.clone()).unwrap();

        let expected = Completion {
            messages: vec![completion.messages[1].clone()],
            ..completion
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn block() {
        let sanitizer = make_sanitizer();

        let completion = Completion {
            model: "llama",
            messages: vec![
                // Block
                Message {
                    role: "user",
                    content: "Their mail is pdk@flex.com and his phone number is +1-212-456-7890",
                },
                Message {
                    role: "user",
                    content: "Their name is PDK",
                },
            ],
            extra: HashMap::default(),
        };

        let actual = sanitizer.sanitize(completion);

        assert!(actual.is_none());
    }
}
