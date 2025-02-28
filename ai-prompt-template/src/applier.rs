// Copyright 2023 Salesforce, Inc. All rights reserved.
use std::{borrow::Cow, collections::HashMap, fmt::Display};

use pdk::logger;

use crate::generated::config::Config;

use std::iter;

#[derive(Debug)]
struct Template<'a> {
    prefix: &'a str,
    needles: Vec<Needle<'a>>,
}

#[derive(Debug)]
struct Needle<'a> {
    variable: &'a str,
    suffix: &'a str,
}

impl<'a> Template<'a> {
    fn parse(input: &'a str) -> Option<Self> {
        let mut parts = input.split("{{");

        let prefix = parts.next()?;
        let needles = parts
            .map(|s| {
                let mut parts = s.split("}}");
                let variable = parts.next()?;
                let suffix = parts.next()?;
                parts
                    .next()
                    .is_none()
                    .then_some(Needle { variable, suffix })
            })
            .collect::<Option<_>>()?;

        Some(Template { prefix, needles })
    }

    fn apply(&self, variables: &HashMap<&'a str, &'a str>) -> TemplateApplication<'a> {
        let replacements = self
            .needles
            .iter()
            .map(|n| {
                (
                    variables.get(n.variable).copied().unwrap_or_default(),
                    n.suffix,
                )
            })
            .flat_map(|(a, b)| iter::once(a).chain(iter::once(b)));

        TemplateApplication {
            parts: iter::once(self.prefix).chain(replacements).collect(),
        }
    }
}

#[derive(Debug)]

pub struct TemplateApplication<'a> {
    parts: Vec<&'a str>,
}

impl<'a> TemplateApplication<'a> {
    pub fn to_bytes(&self) -> Cow<'a, [u8]> {
        match self.parts.as_slice() {
            [] => Cow::Borrowed(&[]),
            [s] => Cow::Borrowed(s.as_bytes()),
            parts => Cow::Owned(
                parts
                    .iter()
                    .flat_map(|s| s.as_bytes().iter().cloned())
                    .collect(),
            ),
        }
    }
}

impl Display for TemplateApplication<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for s in &self.parts {
            write!(f, "{s}")?;
        }
        std::fmt::Result::Ok(())
    }
}

#[derive(Debug)]
pub struct TemplateApplicator<'a> {
    templates: HashMap<&'a str, Template<'a>>,
}

impl<'a> TemplateApplicator<'a> {
    
    pub fn from_config(config: &'a Config) -> Self {
        let templates = config
            .templates
            .iter()
            .flat_map(|c| {
                let name = c.name.as_str();
                let template = Template::parse(&c.template).map(|t| (name, t));
                if template.is_none() {
                    logger::warn!(
                        "Template with name '{name}' was skipped due to incorrect format."
                    )
                }
                template
            })
            .collect();

        Self { templates }
    }

    pub fn apply(
        &self,
        name: &str,
        variables: &HashMap<&'a str, &'a str>,
    ) -> Option<TemplateApplication<'a>> {
        self.templates.get(name).map(|t| t.apply(variables))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::generated::config::{Config, Templates0Config as ConfigTemplate};

    use super::TemplateApplicator;

    #[test]
    fn apply() {
        let config = Config {
            allow_untemplated_requests: false,
            templates: vec![ConfigTemplate {
                name: "default-template".to_string(),
                template: "replacing a {{foo}} with {{bar}} and {{baz}}".to_string(),
            }],
        };

        let applicator = TemplateApplicator::from_config(&config);

        let application = applicator
            .apply(
                "default-template",
                &HashMap::from([("foo", "foo-value"), ("baz", "baz-value")]),
            )
            .expect("application exists");

        assert_eq!(
            application.to_string(),
            "replacing a foo-value with  and baz-value"
        );
    }

    #[test]
    fn apply_without_variables() {
        let config = Config {
            allow_untemplated_requests: false,
            templates: vec![ConfigTemplate {
                name: "default-template".to_string(),
                template: "no variables here".to_string(),
            }],
        };

        let applicator = TemplateApplicator::from_config(&config);

        let application = applicator
            .apply(
                "default-template",
                &HashMap::from([("foo", "foo-value"), ("baz", "baz-value")]),
            )
            .expect("application exists");

        assert_eq!(application.to_string(), "no variables here");
    }
}
