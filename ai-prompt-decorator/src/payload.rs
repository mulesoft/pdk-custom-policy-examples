// Copyright 2023 Salesforce, Inc. All rights reserved.
use crate::generated::config::Config;

use serde::{Deserialize, Serialize};

/// Represents an 'llm/v1/chat' request payload.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payload<'a> {
    #[serde(borrow)]
    messages: Vec<Message<'a>>,
}

/// Represents an 'llm/v1/chat' request message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

/// A context holder for decorating request [Payload]s.
pub struct PayloadDecorator<'a> {
    prepend: Vec<Message<'a>>,
    append: Vec<Message<'a>>,
}

impl<'a> PayloadDecorator<'a> {
    /// Creates a new [PayloadDecorator] from a [Config].
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

    /// Creates a decorated [Payload].
    pub fn decorate(&self, payload: &Payload<'a>) -> Payload<'a> {
        Payload {
            messages: self
                .prepend
                .iter()
                .chain(payload.messages.iter())
                .chain(self.append.iter())
                .cloned()
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generated::config::{Append0Config as Append, Config, Prepend0Config as Prepend};

    use super::{Message, Payload, PayloadDecorator};

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

        let payload = Payload {
            messages: vec![Message {
                role: "user",
                content: "User content",
            }],
        };

        let expected = Payload {
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
        };

        let decorator = PayloadDecorator::from_config(&config);

        let actual = decorator.decorate(&payload);

        assert_eq!(actual, expected);
    }
}
