pub mod schemas;

use crate::anypoint::schemas::{APIMLoginRequest, APIMLoginResponse, LoginPayload, LoginResponse};
use crate::http_utils::CONTENT_TYPE_HEADER;
use pdk::classy::proxy_wasm::types::Status;
use pdk::hl::Uri;
use pdk::metadata::Metadata;
use pdk::policy_context::static_policy_context_cache::StaticPolicyContextCache;
use pdk::{
    hl::{HttpClient, HttpClientError, HttpClientResponse, Service},
    logger,
};
use serde_json::Value;
use std::str::FromStr;
use std::time::Duration;

pub const PLATFORM_TIMEOUT_DURATION: Duration = Duration::from_secs(10);
const PLATFORM_OAUTH_PATH: &str = "/accounts/oauth2/token";
const PLATFORM_API_MANAGER_V1: &str = "/apimanager/api/v1/organizations";
const AUTHORIZATION_PREFIX: &str = "Bearer ";
const ENDPOINT_URL_FIELD: &'static str = "endpointUri";

pub trait PlatformClient {
    fn login(&self) -> impl futures::Future<Output = Result<LoginResponse, HttpClientError>>;

    fn login_apim(
        &self,
        token: &str,
    ) -> impl futures::Future<Output = Result<APIMLoginResponse, HttpClientError>>;

    fn consumer_url(
        &self,
        token: &str,
    ) -> impl futures::Future<Output = Result<Option<String>, HttpClientError>>;
}

pub struct HttpPlatformClient<'a> {
    http_client: &'a HttpClient,
    organization_id: String,
    environment_id: String,
    api_id: String,
    client_id: String,
    client_secret: String,
    service: Service,
    base_path: String,
}

impl<'a> HttpPlatformClient<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(http_client: &'a HttpClient, metadata: &Metadata) -> Self {
        let policy_metadata = StaticPolicyContextCache::read_metadata();
        let environment = policy_metadata.anypoint_environment().unwrap();
        let anypoint = environment.anypoint().unwrap();
        let base_path = anypoint.base_path();
        let sanitized_base_path = match base_path.as_str() {
            "/" => "".to_string(),
            _ => base_path,
        };
        let id = &metadata.api_metadata.id;
        let organization_id = &metadata.platform_metadata.organization_id;
        let environment_id = &metadata.platform_metadata.environment_id;
        Self {
            http_client,
            client_id: anypoint.client_id().to_string(),
            client_secret: anypoint.client_secret().to_string(),
            api_id: id.clone().unwrap(),
            service: Service::new(
                anypoint.service_name(),
                Uri::from_str(anypoint.url()).expect("invalid authority"),
            ),
            base_path: sanitized_base_path,
            organization_id: organization_id.clone(),
            environment_id: environment_id.clone(),
        }
    }
}

impl<'a> PlatformClient for HttpPlatformClient<'a> {
    async fn login(&self) -> Result<LoginResponse, HttpClientError> {
        let login_data = LoginPayload::new(self.client_id.as_str(), self.client_secret.as_str());
        let json = serde_json::to_vec(&login_data).unwrap();
        let path = format!("{}{}", self.base_path, PLATFORM_OAUTH_PATH);
        let headers = vec![(CONTENT_TYPE_HEADER, "application/json")];
        let result = self
            .send_request("POST", path.as_str(), headers, Some(&json))
            .await;
        match result {
            Ok(r) if r.status_code() == 200 => Ok(serde_json::from_slice(r.body())
                .map_err(|_| HttpClientError::Status(Status::InternalFailure))?),
            Ok(r) => {
                logger::warn!(
                    "Not able to login to Core Services Login Response Body: {:?}",
                    String::from_utf8(r.body().to_vec())
                );
                Err(HttpClientError::Status(Status::InternalFailure))
            }
            Err(e) => Err(e),
        }
    }

    async fn login_apim(&self, token: &str) -> Result<APIMLoginResponse, HttpClientError> {
        let headers = vec![(CONTENT_TYPE_HEADER, "application/json")];
        let request = APIMLoginRequest::new(token);
        let json = serde_json::to_vec(&request).unwrap();
        let result = self
            .send_request("POST", "/apiplatform/login", headers, Some(&json))
            .await;
        match result {
            Ok(r) if r.status_code() == 200 => Ok(serde_json::from_slice(r.body())
                .map_err(|_| HttpClientError::Status(Status::InternalFailure))?),
            Ok(r) => {
                logger::warn!(
                    "Not able to login to APIM Status Code: {} Login Response Body: {:?}",
                    r.status_code(),
                    String::from_utf8(r.body().to_vec()),
                );

                Err(HttpClientError::Status(Status::InternalFailure))
            }
            Err(e) => Err(e),
        }
    }

    async fn consumer_url(&self, token: &str) -> Result<Option<String>, HttpClientError> {
        let org_id = &self.organization_id;
        let env_id = &self.environment_id;
        let api_id = &self.api_id;
        let auth_value = format!("{}{}", AUTHORIZATION_PREFIX, token);
        let additional_headers: Vec<(&str, &str)> = vec![("Authorization", auth_value.as_str())];
        let path = format!(
            "{PLATFORM_API_MANAGER_V1}/{org_id}/environments/{env_id}/apis/{}",
            api_id
        );
        let result = self
            .send_request("GET", path.as_str(), additional_headers, None)
            .await?;

        if result.status_code() == 200 {
            // Parse the string into a serde_json::Value
            let parsed: Value =
                serde_json::from_slice(result.body()).expect("Failed to parse JSON");
            if let Some(endpoint_uri) = parsed.get(ENDPOINT_URL_FIELD).and_then(|v| v.as_str()) {
                Ok(Some(endpoint_uri.to_string()))
            } else {
                Ok(None)
            }
        } else {
            Err(HttpClientError::Status(Status::InternalFailure))
        }
    }
}

impl<'a> HttpPlatformClient<'a> {
    async fn send_request(
        &self,
        method: &str,
        path: &str,
        additional_headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
    ) -> Result<HttpClientResponse, HttpClientError> {
        let mut headers = vec![("User-Agent", "Flex")];
        headers.extend(additional_headers);
        self.http_client
            .request(&self.service)
            .headers(headers)
            .body(body.unwrap_or_default())
            .path(path)
            .timeout(PLATFORM_TIMEOUT_DURATION)
            .send(method)
            .await
    }
}
