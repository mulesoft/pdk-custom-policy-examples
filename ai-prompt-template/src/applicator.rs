// Copyright 2023 Salesforce, Inc. All rights reserved.
use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use regex::{Captures, Regex, Replacer};

use crate::generated::config::Config;

/// Searches for a variable expressed as {{var-name}}.
static REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{([a-zA-Z0-9_-]+)\}\}").expect("regex creation"));

/// Replaces variables with the input values.
struct VariableReplacer<'a>(HashMap<&'a str, &'a str>);

impl Replacer for VariableReplacer<'_> {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        let name = &caps[1];
        let replacement = self.0.get(name).cloned().unwrap_or_else(|| &caps[0]);
        dst.push_str(replacement);
    }
}

/// Stores templates indexed by name, and applies variables on them.
pub struct TemplateApplicator<'a> {
    templates: HashMap<&'a str, &'a str>,
}

impl<'a> TemplateApplicator<'a> {
    /// Creates a new [TemplateApplicator] from [Config].
    pub fn from_config(config: &'a Config) -> Self {
        let templates = config
            .templates
            .iter()
            .map(|c| (c.name.as_str(), c.template.as_str()))
            .collect();

        Self { templates }
    }

    /// Applies input variables on templates.
    /// Retorns [None] if there is no template for the requested `name`.
    pub fn apply(&self, name: &str, variables: HashMap<&'a str, &'a str>) -> Option<Cow<'a, str>> {
        self.templates
            .get(name)
            .map(|template| REGEX.replace_all(template, VariableReplacer(variables)))
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
                HashMap::from([("foo", "foo-value"), ("baz", "baz-value")]),
            )
            .expect("application exists");

        // bar is skipped since it is not present
        assert_eq!(
            application,
            "replacing a foo-value with {{bar}} and baz-value"
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
                HashMap::from([("foo", "foo-value"), ("baz", "baz-value")]),
            )
            .expect("application exists");

        assert_eq!(application, "no variables here");
    }
}
