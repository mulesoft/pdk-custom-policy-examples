use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIMLoginResponse {
    token: String,
}

impl APIMLoginResponse {
    pub fn get_token(&self) -> &str {
        self.token.as_str()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIMLoginRequest {
    token: String,
}

impl APIMLoginRequest {
    pub fn new(token: &str) -> APIMLoginRequest {
        APIMLoginRequest {
            token: token.to_string(),
        }
    }
}
impl LoginResponse {
    pub fn get_token(&self) -> &str {
        self.access_token.as_deref().unwrap_or_default()
    }

    pub fn get_type(&self) -> &str {
        self.token_type.as_deref().unwrap_or_default()
    }
}

#[derive(Serialize, Debug)]
pub struct LoginPayload {
    grant_type: String,
    client_id: String,
    client_secret: String,
}

impl LoginPayload {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            grant_type: String::from("client_credentials"),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }
}
