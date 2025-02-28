// Copyright 2023 Salesforce, Inc. All rights reserved.
use crate::{
    generated::config::Config,
    openai::{Completion, Message},
};

/// A context holder for decorating [Completion]s.
pub struct CompletionDecorator<'a> {
    prepend: Vec<Message<'a>>,
    append: Vec<Message<'a>>,
}

impl<'a> CompletionDecorator<'a> {
    /// Creates a new [CompletionDecorator] from a [Config].
    pub fn from_config(config: &'a Config) -> Self {
        Self {
            prepend: config
                .prepend
                .iter()
                .map(|p| Message {
                    role: &p.role,
                    content: &p.content,
                })
                .collect(),
            append: config
                .append
                .iter()
                .map(|a| Message {
                    role: &a.role,
                    content: &a.content,
                })
                .collect(),
        }
    }

    /// Creates a decorated [Completion].
    pub fn decorate(&self, completion: Completion<'a>) -> Completion<'a> {
        Completion {
            messages: self
                .prepend
                .iter()
                .cloned()
                .chain(completion.messages)
                .chain(self.append.iter().cloned())
                .collect(),
            ..completion
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::generated::config::{Append0Config as Append, Config, Prepend0Config as Prepend};

    use super::{Completion, CompletionDecorator, Message};

    #[test]
    fn decorate() {
        let config = Config {
            prepend: vec![
                Prepend {
                    role: "system".to_string(),
                    content: "prepend content 0.".to_string(),
                },
                Prepend {
                    role: "user".to_string(),
                    content: "prepend content 1.".to_string(),
                },
            ],
            append: vec![Append {
                role: "user".to_string(),
                content: "append content 0.".to_string(),
            }],
        };

        let payload = Completion {
            model: "llama",
            messages: vec![Message {
                role: "user",
                content: "User content",
            }],
            extra: HashMap::default(),
        };

        let expected = Completion {
            model: "llama",
            messages: vec![
                Message {
                    role: &config.prepend[0].role,
                    content: &config.prepend[0].content,
                },
                Message {
                    role: &config.prepend[1].role,
                    content: &config.prepend[1].content,
                },
                payload.messages[0].clone(),
                Message {
                    role: &config.append[0].role,
                    content: &config.append[0].content,
                },
            ],
            extra: HashMap::default(),
        };

        let decorator = CompletionDecorator::from_config(&config);

        let actual = decorator.decorate(payload);

        assert_eq!(actual, expected);
    }
}
