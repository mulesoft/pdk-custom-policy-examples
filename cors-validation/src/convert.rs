// Copyright 2023 Salesforce, Inc. All rights reserved.
use anyhow::Result;
use pdk::cors;

use crate::generated::config::AllowedMethods0Config as AllowedMethod;
use crate::generated::config::Config;
use crate::generated::config::OriginGroups0Config as OriginGroup;

/// This module implements conversions from policy [`Config`] into a [`cors::Configuration`].

impl Config {
    /// Translates the policy [`Config`] into a [`cors::Configuration`].
    pub fn into_cors(self) -> Result<cors::Configuration<'static>> {
        let origin_groups: Result<Vec<_>> = self
            .origin_groups
            .into_iter()
            .map(OriginGroup::into_cors)
            .collect();

        let config = cors::Configuration::builder()
            .public_resource(self.public_resource)
            .support_credentials(self.support_credentials)
            .origin_groups(origin_groups?)
            .build()?;

        Ok(config)
    }
}

impl OriginGroup {
    fn into_cors(self) -> Result<cors::OriginGroup<'static>> {
        let allowed_methods: Result<Vec<_>> = self
            .allowed_methods
            .into_iter()
            .map(AllowedMethod::into_cors)
            .collect();

        let origin_group = cors::OriginGroup::builder()
            .origin_group_name(self.name)
            .plain_origins(self.origins)
            .access_control_max_age(self.access_control_max_age as u32)
            .allowed_methods(allowed_methods?)
            .headers(self.headers)
            .exposed_headers(self.exposed_headers)
            .build()?;

        Ok(origin_group)
    }
}

impl AllowedMethod {
    fn into_cors(self) -> Result<cors::AllowedMethod> {
        let method = cors::AllowedMethod::builder()
            .method_name(self.method_name)
            .allowed(self.allowed)
            .build()?;
        Ok(method)
    }
}
