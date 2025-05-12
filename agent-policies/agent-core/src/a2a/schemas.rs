#![allow(clippy::redundant_closure_call)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::clone_on_copy)]

#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
#[doc = "JSON Schema for A2A Protocol"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"A2A Protocol Schema\","]
#[doc = "  \"description\": \"JSON Schema for A2A Protocol\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct A2aProtocolSchema(pub ::serde_json::Value);
impl ::std::ops::Deref for A2aProtocolSchema {
    type Target = ::serde_json::Value;
    fn deref(&self) -> &::serde_json::Value {
        &self.0
    }
}
impl ::std::convert::From<A2aProtocolSchema> for ::serde_json::Value {
    fn from(value: A2aProtocolSchema) -> Self {
        value.0
    }
}
impl ::std::convert::From<&A2aProtocolSchema> for A2aProtocolSchema {
    fn from(value: &A2aProtocolSchema) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<::serde_json::Value> for A2aProtocolSchema {
    fn from(value: ::serde_json::Value) -> Self {
        Self(value)
    }
}
#[doc = "`A2aRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"A2ARequest\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/SendTaskRequest\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GetTaskRequest\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/CancelTaskRequest\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/SetTaskPushNotificationRequest\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/GetTaskPushNotificationRequest\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TaskResubscriptionRequest\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum A2aRequest {
    SendTaskRequest(SendTaskRequest),
    GetTaskRequest(GetTaskRequest),
    CancelTaskRequest(CancelTaskRequest),
    SetTaskPushNotificationRequest(SetTaskPushNotificationRequest),
    GetTaskPushNotificationRequest(GetTaskPushNotificationRequest),
    TaskResubscriptionRequest(TaskResubscriptionRequest),
}
impl ::std::convert::From<&Self> for A2aRequest {
    fn from(value: &A2aRequest) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<SendTaskRequest> for A2aRequest {
    fn from(value: SendTaskRequest) -> Self {
        Self::SendTaskRequest(value)
    }
}
impl ::std::convert::From<GetTaskRequest> for A2aRequest {
    fn from(value: GetTaskRequest) -> Self {
        Self::GetTaskRequest(value)
    }
}
impl ::std::convert::From<CancelTaskRequest> for A2aRequest {
    fn from(value: CancelTaskRequest) -> Self {
        Self::CancelTaskRequest(value)
    }
}
impl ::std::convert::From<SetTaskPushNotificationRequest> for A2aRequest {
    fn from(value: SetTaskPushNotificationRequest) -> Self {
        Self::SetTaskPushNotificationRequest(value)
    }
}
impl ::std::convert::From<GetTaskPushNotificationRequest> for A2aRequest {
    fn from(value: GetTaskPushNotificationRequest) -> Self {
        Self::GetTaskPushNotificationRequest(value)
    }
}
impl ::std::convert::From<TaskResubscriptionRequest> for A2aRequest {
    fn from(value: TaskResubscriptionRequest) -> Self {
        Self::TaskResubscriptionRequest(value)
    }
}
#[doc = "`AgentAuthentication`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AgentAuthentication\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"schemes\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"credentials\": {"]
#[doc = "      \"title\": \"Credentials\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"schemes\": {"]
#[doc = "      \"title\": \"Schemes\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct AgentAuthentication {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub credentials: ::std::option::Option<::std::string::String>,
    pub schemes: ::std::vec::Vec<::std::string::String>,
}
impl ::std::convert::From<&AgentAuthentication> for AgentAuthentication {
    fn from(value: &AgentAuthentication) -> Self {
        value.clone()
    }
}
#[doc = "`AgentCapabilities`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AgentCapabilities\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"pushNotifications\": {"]
#[doc = "      \"title\": \"PushNotifications\","]
#[doc = "      \"default\": false,"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"stateTransitionHistory\": {"]
#[doc = "      \"title\": \"Statetransitionhistory\","]
#[doc = "      \"default\": false,"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"streaming\": {"]
#[doc = "      \"title\": \"Streaming\","]
#[doc = "      \"default\": false,"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct AgentCapabilities {
    #[serde(rename = "pushNotifications", default)]
    pub push_notifications: bool,
    #[serde(rename = "stateTransitionHistory", default)]
    pub state_transition_history: bool,
    #[serde(default)]
    pub streaming: bool,
}
impl ::std::convert::From<&AgentCapabilities> for AgentCapabilities {
    fn from(value: &AgentCapabilities) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            push_notifications: Default::default(),
            state_transition_history: Default::default(),
            streaming: Default::default(),
        }
    }
}
#[doc = "`AgentCard`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AgentCard\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"capabilities\","]
#[doc = "    \"name\","]
#[doc = "    \"skills\","]
#[doc = "    \"url\","]
#[doc = "    \"version\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"authentication\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/AgentAuthentication\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"capabilities\": {"]
#[doc = "      \"$ref\": \"#/$defs/AgentCapabilities\""]
#[doc = "    },"]
#[doc = "    \"defaultInputModes\": {"]
#[doc = "      \"title\": \"Defaultinputmodes\","]
#[doc = "      \"default\": ["]
#[doc = "        \"text\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"defaultOutputModes\": {"]
#[doc = "      \"title\": \"Defaultoutputmodes\","]
#[doc = "      \"default\": ["]
#[doc = "        \"text\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"description\": {"]
#[doc = "      \"title\": \"Description\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"documentationUrl\": {"]
#[doc = "      \"title\": \"Documentationurl\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"title\": \"Name\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"provider\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/AgentProvider\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"skills\": {"]
#[doc = "      \"title\": \"Skills\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/AgentSkill\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"url\": {"]
#[doc = "      \"title\": \"Url\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"version\": {"]
#[doc = "      \"title\": \"Version\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct AgentCard {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub authentication: ::std::option::Option<AgentAuthentication>,
    pub capabilities: AgentCapabilities,
    #[serde(
        rename = "defaultInputModes",
        default = "defaults::agent_card_default_input_modes"
    )]
    pub default_input_modes: ::std::vec::Vec<::std::string::String>,
    #[serde(
        rename = "defaultOutputModes",
        default = "defaults::agent_card_default_output_modes"
    )]
    pub default_output_modes: ::std::vec::Vec<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub description: ::std::option::Option<::std::string::String>,
    #[serde(
        rename = "documentationUrl",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub documentation_url: ::std::option::Option<::std::string::String>,
    pub name: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub provider: ::std::option::Option<AgentProvider>,
    pub skills: ::std::vec::Vec<AgentSkill>,
    pub url: ::std::string::String,
    pub version: ::std::string::String,
}
impl ::std::convert::From<&AgentCard> for AgentCard {
    fn from(value: &AgentCard) -> Self {
        value.clone()
    }
}
#[doc = "`AgentProvider`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AgentProvider\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"organization\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"organization\": {"]
#[doc = "      \"title\": \"Organization\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"url\": {"]
#[doc = "      \"title\": \"Url\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct AgentProvider {
    pub organization: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub url: ::std::option::Option<::std::string::String>,
}
impl ::std::convert::From<&AgentProvider> for AgentProvider {
    fn from(value: &AgentProvider) -> Self {
        value.clone()
    }
}
#[doc = "`AgentSkill`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AgentSkill\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"name\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"description\": {"]
#[doc = "      \"title\": \"Description\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"examples\": {"]
#[doc = "      \"title\": \"Examples\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"inputModes\": {"]
#[doc = "      \"title\": \"Inputmodes\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"title\": \"Name\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"outputModes\": {"]
#[doc = "      \"title\": \"Outputmodes\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"tags\": {"]
#[doc = "      \"title\": \"Tags\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"type\": \"string\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct AgentSkill {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub description: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub examples: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
    pub id: ::std::string::String,
    #[serde(
        rename = "inputModes",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub input_modes: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
    pub name: ::std::string::String,
    #[serde(
        rename = "outputModes",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub output_modes: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub tags: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
}
impl ::std::convert::From<&AgentSkill> for AgentSkill {
    fn from(value: &AgentSkill) -> Self {
        value.clone()
    }
}
#[doc = "`Artifact`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Artifact\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"parts\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"append\": {"]
#[doc = "      \"title\": \"Append\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"description\": {"]
#[doc = "      \"title\": \"Description\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"index\": {"]
#[doc = "      \"title\": \"Index\","]
#[doc = "      \"default\": 0,"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"lastChunk\": {"]
#[doc = "      \"title\": \"LastChunk\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"title\": \"Name\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"parts\": {"]
#[doc = "      \"title\": \"Parts\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/Part\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Artifact {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub append: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub description: ::std::option::Option<::std::string::String>,
    #[serde(default)]
    pub index: i64,
    #[serde(
        rename = "lastChunk",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub last_chunk: ::std::option::Option<bool>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub name: ::std::option::Option<::std::string::String>,
    pub parts: ::std::vec::Vec<Part>,
}
impl ::std::convert::From<&Artifact> for Artifact {
    fn from(value: &Artifact) -> Self {
        value.clone()
    }
}
#[doc = "`AuthenticationInfo`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AuthenticationInfo\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"schemes\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"credentials\": {"]
#[doc = "      \"title\": \"Credentials\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"schemes\": {"]
#[doc = "      \"title\": \"Schemes\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"type\": \"string\""]
#[doc = "      }"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": {}"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct AuthenticationInfo {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub credentials: ::std::option::Option<::std::string::String>,
    pub schemes: ::std::vec::Vec<::std::string::String>,
    #[serde(flatten)]
    pub extra: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
}
impl ::std::convert::From<&AuthenticationInfo> for AuthenticationInfo {
    fn from(value: &AuthenticationInfo) -> Self {
        value.clone()
    }
}
#[doc = "`CancelTaskRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"CancelTaskRequest\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"method\","]
#[doc = "    \"params\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"method\": {"]
#[doc = "      \"title\": \"Method\","]
#[doc = "      \"default\": \"tasks/cancel\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"tasks/cancel\""]
#[doc = "    },"]
#[doc = "    \"params\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskIdParams\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct CancelTaskRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::cancel_task_request_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub params: TaskIdParams,
}
impl ::std::convert::From<&CancelTaskRequest> for CancelTaskRequest {
    fn from(value: &CancelTaskRequest) -> Self {
        value.clone()
    }
}
#[doc = "`CancelTaskResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"CancelTaskResponse\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/JSONRPCError\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"result\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/Task\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct CancelTaskResponse {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<JsonrpcError>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::cancel_task_response_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub result: ::std::option::Option<Task>,
}
impl ::std::convert::From<&CancelTaskResponse> for CancelTaskResponse {
    fn from(value: &CancelTaskResponse) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for CancelTaskResponse {
    fn default() -> Self {
        Self {
            error: Default::default(),
            id: Default::default(),
            jsonrpc: defaults::cancel_task_response_jsonrpc(),
            result: Default::default(),
        }
    }
}
#[doc = "`DataPart`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"DataPart\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"data\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"additionalProperties\": {}"]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"title\": \"Type\","]
#[doc = "      \"description\": \"Type of the part\","]
#[doc = "      \"default\": \"data\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"data\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"data\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct DataPart {
    pub data: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[doc = "Type of the part"]
    #[serde(rename = "type")]
    pub type_: ::std::string::String,
}
impl ::std::convert::From<&DataPart> for DataPart {
    fn from(value: &DataPart) -> Self {
        value.clone()
    }
}
#[doc = "Represents the content of a file, either as base64 encoded bytes or a URI.\n\nEnsures that either 'bytes' or 'uri' is provided, but not both."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"FileContent\","]
#[doc = "  \"description\": \"Represents the content of a file, either as base64 encoded bytes or a URI.\\n\\nEnsures that either 'bytes' or 'uri' is provided, but not both.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"bytes\": {"]
#[doc = "      \"title\": \"Bytes\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"mimeType\": {"]
#[doc = "      \"title\": \"Mimetype\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"name\": {"]
#[doc = "      \"title\": \"Name\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"uri\": {"]
#[doc = "      \"title\": \"Uri\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct FileContent {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub bytes: ::std::option::Option<::std::string::String>,
    #[serde(
        rename = "mimeType",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub mime_type: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub name: ::std::option::Option<::std::string::String>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub uri: ::std::option::Option<::std::string::String>,
}
impl ::std::convert::From<&FileContent> for FileContent {
    fn from(value: &FileContent) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for FileContent {
    fn default() -> Self {
        Self {
            bytes: Default::default(),
            mime_type: Default::default(),
            name: Default::default(),
            uri: Default::default(),
        }
    }
}
#[doc = "`FilePart`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"FilePart\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"file\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"file\": {"]
#[doc = "      \"$ref\": \"#/$defs/FileContent\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"title\": \"Type\","]
#[doc = "      \"description\": \"Type of the part\","]
#[doc = "      \"default\": \"file\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"file\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"file\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct FilePart {
    pub file: FileContent,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[doc = "Type of the part"]
    #[serde(rename = "type")]
    pub type_: ::std::string::String,
}
impl ::std::convert::From<&FilePart> for FilePart {
    fn from(value: &FilePart) -> Self {
        value.clone()
    }
}
#[doc = "`GetTaskPushNotificationRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GetTaskPushNotificationRequest\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"method\","]
#[doc = "    \"params\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"method\": {"]
#[doc = "      \"title\": \"Method\","]
#[doc = "      \"default\": \"tasks/pushNotification/get\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"tasks/pushNotification/get\""]
#[doc = "    },"]
#[doc = "    \"params\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskIdParams\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct GetTaskPushNotificationRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::get_task_push_notification_request_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub params: TaskIdParams,
}
impl ::std::convert::From<&GetTaskPushNotificationRequest> for GetTaskPushNotificationRequest {
    fn from(value: &GetTaskPushNotificationRequest) -> Self {
        value.clone()
    }
}
#[doc = "`GetTaskPushNotificationResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GetTaskPushNotificationResponse\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/JSONRPCError\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"result\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/TaskPushNotificationConfig\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct GetTaskPushNotificationResponse {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<JsonrpcError>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::get_task_push_notification_response_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub result: ::std::option::Option<TaskPushNotificationConfig>,
}
impl ::std::convert::From<&GetTaskPushNotificationResponse> for GetTaskPushNotificationResponse {
    fn from(value: &GetTaskPushNotificationResponse) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for GetTaskPushNotificationResponse {
    fn default() -> Self {
        Self {
            error: Default::default(),
            id: Default::default(),
            jsonrpc: defaults::get_task_push_notification_response_jsonrpc(),
            result: Default::default(),
        }
    }
}
#[doc = "`GetTaskRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GetTaskRequest\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"method\","]
#[doc = "    \"params\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"method\": {"]
#[doc = "      \"title\": \"Method\","]
#[doc = "      \"default\": \"tasks/get\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"tasks/get\""]
#[doc = "    },"]
#[doc = "    \"params\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskQueryParams\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct GetTaskRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::get_task_request_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub params: TaskQueryParams,
}
impl ::std::convert::From<&GetTaskRequest> for GetTaskRequest {
    fn from(value: &GetTaskRequest) -> Self {
        value.clone()
    }
}
#[doc = "`GetTaskResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"GetTaskResponse\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/JSONRPCError\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"result\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/Task\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct GetTaskResponse {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<JsonrpcError>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::get_task_response_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub result: ::std::option::Option<Task>,
}
impl ::std::convert::From<&GetTaskResponse> for GetTaskResponse {
    fn from(value: &GetTaskResponse) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for GetTaskResponse {
    fn default() -> Self {
        Self {
            error: Default::default(),
            id: Default::default(),
            jsonrpc: defaults::get_task_response_jsonrpc(),
            result: Default::default(),
        }
    }
}
#[doc = "`Id`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Id\","]
#[doc = "  \"anyOf\": ["]
#[doc = "    {"]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Id {
    Variant0(i64),
    Variant1(::std::string::String),
    Variant2,
}
impl ::std::convert::From<&Self> for Id {
    fn from(value: &Id) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<i64> for Id {
    fn from(value: i64) -> Self {
        Self::Variant0(value)
    }
}
#[doc = "`InternalError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"InternalError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32603,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32603"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32603"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"Internal error\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"Internal error\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"Internal error\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct InternalError {
    #[doc = "Error code"]
    pub code: i64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub data: ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&InternalError> for InternalError {
    fn from(value: &InternalError) -> Self {
        value.clone()
    }
}
#[doc = "`InvalidParamsError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"InvalidParamsError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32602,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32602"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32602"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"Invalid parameters\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"Invalid parameters\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"Invalid parameters\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct InvalidParamsError {
    #[doc = "Error code"]
    pub code: i64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub data: ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&InvalidParamsError> for InvalidParamsError {
    fn from(value: &InvalidParamsError) -> Self {
        value.clone()
    }
}
#[doc = "`InvalidRequestError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"InvalidRequestError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32600,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32600"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32600"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"Request payload validation error\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"Request payload validation error\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"Request payload validation error\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct InvalidRequestError {
    #[doc = "Error code"]
    pub code: i64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub data: ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&InvalidRequestError> for InvalidRequestError {
    fn from(value: &InvalidRequestError) -> Self {
        value.clone()
    }
}
#[doc = "`JsonParseError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"JSONParseError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32700,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32700"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32700"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"Invalid JSON payload\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"Invalid JSON payload\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"Invalid JSON payload\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct JsonParseError {
    #[doc = "Error code"]
    pub code: i64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub data: ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&JsonParseError> for JsonParseError {
    fn from(value: &JsonParseError) -> Self {
        value.clone()
    }
}
#[doc = "`JsonrpcError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"JSONRPCError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"type\": \"integer\""]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct JsonrpcError {
    pub code: i64,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub data: ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    pub message: ::std::string::String,
}
impl ::std::convert::From<&JsonrpcError> for JsonrpcError {
    fn from(value: &JsonrpcError) -> Self {
        value.clone()
    }
}
#[doc = "`JsonrpcMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"JSONRPCMessage\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct JsonrpcMessage {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::jsonrpc_message_jsonrpc")]
    pub jsonrpc: ::std::string::String,
}
impl ::std::convert::From<&JsonrpcMessage> for JsonrpcMessage {
    fn from(value: &JsonrpcMessage) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for JsonrpcMessage {
    fn default() -> Self {
        Self {
            id: Default::default(),
            jsonrpc: defaults::jsonrpc_message_jsonrpc(),
        }
    }
}
#[doc = "`JsonrpcRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"JSONRPCRequest\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"method\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"method\": {"]
#[doc = "      \"title\": \"Method\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"params\": {"]
#[doc = "      \"title\": \"Params\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct JsonrpcRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::jsonrpc_request_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub params:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
}
impl ::std::convert::From<&JsonrpcRequest> for JsonrpcRequest {
    fn from(value: &JsonrpcRequest) -> Self {
        value.clone()
    }
}
#[doc = "`JsonrpcResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"JSONRPCResponse\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/JSONRPCError\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"result\": {"]
#[doc = "      \"title\": \"Result\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct JsonrpcResponse {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<JsonrpcError>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::jsonrpc_response_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub result:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
}
impl ::std::convert::From<&JsonrpcResponse> for JsonrpcResponse {
    fn from(value: &JsonrpcResponse) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for JsonrpcResponse {
    fn default() -> Self {
        Self {
            error: Default::default(),
            id: Default::default(),
            jsonrpc: defaults::jsonrpc_response_jsonrpc(),
            result: Default::default(),
        }
    }
}
#[doc = "`Message`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Message\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"parts\","]
#[doc = "    \"role\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"parts\": {"]
#[doc = "      \"title\": \"Parts\","]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/Part\""]
#[doc = "      }"]
#[doc = "    },"]
#[doc = "    \"role\": {"]
#[doc = "      \"title\": \"Role\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"enum\": ["]
#[doc = "        \"user\","]
#[doc = "        \"agent\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Message {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    pub parts: ::std::vec::Vec<Part>,
    pub role: Role,
}
impl ::std::convert::From<&Message> for Message {
    fn from(value: &Message) -> Self {
        value.clone()
    }
}
#[doc = "`MethodNotFoundError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"MethodNotFoundError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"data\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32601,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32601"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32601"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"const\": null"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"Method not found\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"Method not found\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"Method not found\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct MethodNotFoundError {
    #[doc = "Error code"]
    pub code: i64,
    pub data: ::serde_json::Value,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&MethodNotFoundError> for MethodNotFoundError {
    fn from(value: &MethodNotFoundError) -> Self {
        value.clone()
    }
}
#[doc = "`Part`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Part\","]
#[doc = "  \"anyOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TextPart\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/FilePart\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/DataPart\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Part {
    TextPart(TextPart),
    FilePart(FilePart),
    DataPart(DataPart),
}
impl ::std::convert::From<&Self> for Part {
    fn from(value: &Part) -> Self {
        value.clone()
    }
}
impl ::std::convert::From<TextPart> for Part {
    fn from(value: TextPart) -> Self {
        Self::TextPart(value)
    }
}
impl ::std::convert::From<FilePart> for Part {
    fn from(value: FilePart) -> Self {
        Self::FilePart(value)
    }
}
impl ::std::convert::From<DataPart> for Part {
    fn from(value: DataPart) -> Self {
        Self::DataPart(value)
    }
}
#[doc = "`PushNotificationConfig`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"PushNotificationConfig\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"url\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"authentication\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/AuthenticationInfo\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"token\": {"]
#[doc = "      \"title\": \"Token\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"url\": {"]
#[doc = "      \"title\": \"Url\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct PushNotificationConfig {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub authentication: ::std::option::Option<AuthenticationInfo>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub token: ::std::option::Option<::std::string::String>,
    pub url: ::std::string::String,
}
impl ::std::convert::From<&PushNotificationConfig> for PushNotificationConfig {
    fn from(value: &PushNotificationConfig) -> Self {
        value.clone()
    }
}
#[doc = "`PushNotificationNotSupportedError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"PushNotificationNotSupportedError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"data\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32003,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32003"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32003"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"const\": null"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"Push Notification is not supported\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"Push Notification is not supported\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"Push Notification is not supported\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct PushNotificationNotSupportedError {
    #[doc = "Error code"]
    pub code: i64,
    pub data: ::serde_json::Value,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&PushNotificationNotSupportedError>
    for PushNotificationNotSupportedError
{
    fn from(value: &PushNotificationNotSupportedError) -> Self {
        value.clone()
    }
}
#[doc = "`Role`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Role\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"user\","]
#[doc = "    \"agent\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "agent")]
    Agent,
}
impl ::std::convert::From<&Self> for Role {
    fn from(value: &Role) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for Role {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::User => write!(f, "user"),
            Self::Agent => write!(f, "agent"),
        }
    }
}
impl ::std::str::FromStr for Role {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "user" => Ok(Self::User),
            "agent" => Ok(Self::Agent),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Role {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Role {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Role {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`SendTaskRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"SendTaskRequest\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"method\","]
#[doc = "    \"params\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"method\": {"]
#[doc = "      \"title\": \"Method\","]
#[doc = "      \"default\": \"tasks/send\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"tasks/send\""]
#[doc = "    },"]
#[doc = "    \"params\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskSendParams\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct SendTaskRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::send_task_request_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub params: TaskSendParams,
}
impl ::std::convert::From<&SendTaskRequest> for SendTaskRequest {
    fn from(value: &SendTaskRequest) -> Self {
        value.clone()
    }
}
#[doc = "`SendTaskResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"SendTaskResponse\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/JSONRPCError\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"result\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/Task\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct SendTaskResponse {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<JsonrpcError>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::send_task_response_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub result: ::std::option::Option<Task>,
}
impl ::std::convert::From<&SendTaskResponse> for SendTaskResponse {
    fn from(value: &SendTaskResponse) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for SendTaskResponse {
    fn default() -> Self {
        Self {
            error: Default::default(),
            id: Default::default(),
            jsonrpc: defaults::send_task_response_jsonrpc(),
            result: Default::default(),
        }
    }
}
#[doc = "`SendTaskStreamingRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"SendTaskStreamingRequest\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"method\","]
#[doc = "    \"params\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"method\": {"]
#[doc = "      \"title\": \"Method\","]
#[doc = "      \"default\": \"tasks/sendSubscribe\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"tasks/sendSubscribe\""]
#[doc = "    },"]
#[doc = "    \"params\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskSendParams\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct SendTaskStreamingRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::send_task_streaming_request_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub params: TaskSendParams,
}
impl ::std::convert::From<&SendTaskStreamingRequest> for SendTaskStreamingRequest {
    fn from(value: &SendTaskStreamingRequest) -> Self {
        value.clone()
    }
}
#[doc = "`SendTaskStreamingResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"SendTaskStreamingResponse\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/JSONRPCError\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"result\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/TaskStatusUpdateEvent\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/TaskArtifactUpdateEvent\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct SendTaskStreamingResponse {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<JsonrpcError>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::send_task_streaming_response_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    #[serde(default = "defaults::send_task_streaming_response_result")]
    pub result: SendTaskStreamingResponseResult,
}
impl ::std::convert::From<&SendTaskStreamingResponse> for SendTaskStreamingResponse {
    fn from(value: &SendTaskStreamingResponse) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for SendTaskStreamingResponse {
    fn default() -> Self {
        Self {
            error: Default::default(),
            id: Default::default(),
            jsonrpc: defaults::send_task_streaming_response_jsonrpc(),
            result: defaults::send_task_streaming_response_result(),
        }
    }
}
#[doc = "`SendTaskStreamingResponseResult`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"default\": null,"]
#[doc = "  \"anyOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TaskStatusUpdateEvent\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TaskArtifactUpdateEvent\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum SendTaskStreamingResponseResult {
    Variant0(TaskStatusUpdateEvent),
    Variant1(TaskArtifactUpdateEvent),
    Variant2,
}
impl ::std::convert::From<&Self> for SendTaskStreamingResponseResult {
    fn from(value: &SendTaskStreamingResponseResult) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for SendTaskStreamingResponseResult {
    fn default() -> Self {
        SendTaskStreamingResponseResult::Variant2
    }
}
impl ::std::convert::From<TaskStatusUpdateEvent> for SendTaskStreamingResponseResult {
    fn from(value: TaskStatusUpdateEvent) -> Self {
        Self::Variant0(value)
    }
}
impl ::std::convert::From<TaskArtifactUpdateEvent> for SendTaskStreamingResponseResult {
    fn from(value: TaskArtifactUpdateEvent) -> Self {
        Self::Variant1(value)
    }
}
#[doc = "`SetTaskPushNotificationRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"SetTaskPushNotificationRequest\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"method\","]
#[doc = "    \"params\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"method\": {"]
#[doc = "      \"title\": \"Method\","]
#[doc = "      \"default\": \"tasks/pushNotification/set\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"tasks/pushNotification/set\""]
#[doc = "    },"]
#[doc = "    \"params\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskPushNotificationConfig\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct SetTaskPushNotificationRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::set_task_push_notification_request_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub params: TaskPushNotificationConfig,
}
impl ::std::convert::From<&SetTaskPushNotificationRequest> for SetTaskPushNotificationRequest {
    fn from(value: &SetTaskPushNotificationRequest) -> Self {
        value.clone()
    }
}
#[doc = "`SetTaskPushNotificationResponse`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"SetTaskPushNotificationResponse\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"properties\": {"]
#[doc = "    \"error\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/JSONRPCError\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"result\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/TaskPushNotificationConfig\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct SetTaskPushNotificationResponse {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub error: ::std::option::Option<JsonrpcError>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::set_task_push_notification_response_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub result: ::std::option::Option<TaskPushNotificationConfig>,
}
impl ::std::convert::From<&SetTaskPushNotificationResponse> for SetTaskPushNotificationResponse {
    fn from(value: &SetTaskPushNotificationResponse) -> Self {
        value.clone()
    }
}
impl ::std::default::Default for SetTaskPushNotificationResponse {
    fn default() -> Self {
        Self {
            error: Default::default(),
            id: Default::default(),
            jsonrpc: defaults::set_task_push_notification_response_jsonrpc(),
            result: Default::default(),
        }
    }
}
#[doc = "`Task`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Task\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"status\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"artifacts\": {"]
#[doc = "      \"title\": \"Artifacts\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/Artifact\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"history\": {"]
#[doc = "      \"title\": \"History\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/Message\""]
#[doc = "          }"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"title\": \"Sessionid\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskStatus\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct Task {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub artifacts: ::std::option::Option<::std::vec::Vec<Artifact>>,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub history: ::std::option::Option<::std::vec::Vec<Message>>,
    pub id: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[serde(
        rename = "sessionId",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub session_id: ::std::option::Option<::std::string::String>,
    pub status: TaskStatus,
}
impl ::std::convert::From<&Task> for Task {
    fn from(value: &Task) -> Self {
        value.clone()
    }
}
#[doc = "`TaskArtifactUpdateEvent`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskArtifactUpdateEvent\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"artifact\","]
#[doc = "    \"id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"artifact\": {"]
#[doc = "      \"$ref\": \"#/$defs/Artifact\""]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskArtifactUpdateEvent {
    pub artifact: Artifact,
    pub id: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
}
impl ::std::convert::From<&TaskArtifactUpdateEvent> for TaskArtifactUpdateEvent {
    fn from(value: &TaskArtifactUpdateEvent) -> Self {
        value.clone()
    }
}
#[doc = "`TaskIdParams`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskIdParams\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskIdParams {
    pub id: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
}
impl ::std::convert::From<&TaskIdParams> for TaskIdParams {
    fn from(value: &TaskIdParams) -> Self {
        value.clone()
    }
}
#[doc = "`TaskNotCancelableError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskNotCancelableError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"data\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32002,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32002"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32002"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"const\": null"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"Task cannot be canceled\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"Task cannot be canceled\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"Task cannot be canceled\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskNotCancelableError {
    #[doc = "Error code"]
    pub code: i64,
    pub data: ::serde_json::Value,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&TaskNotCancelableError> for TaskNotCancelableError {
    fn from(value: &TaskNotCancelableError) -> Self {
        value.clone()
    }
}
#[doc = "`TaskNotFoundError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskNotFoundError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"data\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32001,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32001"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32001"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"const\": null"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"Task not found\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"Task not found\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"Task not found\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskNotFoundError {
    #[doc = "Error code"]
    pub code: i64,
    pub data: ::serde_json::Value,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&TaskNotFoundError> for TaskNotFoundError {
    fn from(value: &TaskNotFoundError) -> Self {
        value.clone()
    }
}
#[doc = "`TaskPushNotificationConfig`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskPushNotificationConfig\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"pushNotificationConfig\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"pushNotificationConfig\": {"]
#[doc = "      \"$ref\": \"#/$defs/PushNotificationConfig\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskPushNotificationConfig {
    pub id: ::std::string::String,
    #[serde(rename = "pushNotificationConfig")]
    pub push_notification_config: PushNotificationConfig,
}
impl ::std::convert::From<&TaskPushNotificationConfig> for TaskPushNotificationConfig {
    fn from(value: &TaskPushNotificationConfig) -> Self {
        value.clone()
    }
}
#[doc = "`TaskQueryParams`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskQueryParams\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"historyLength\": {"]
#[doc = "      \"title\": \"HistoryLength\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskQueryParams {
    #[serde(
        rename = "historyLength",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub history_length: ::std::option::Option<i64>,
    pub id: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
}
impl ::std::convert::From<&TaskQueryParams> for TaskQueryParams {
    fn from(value: &TaskQueryParams) -> Self {
        value.clone()
    }
}
#[doc = "`TaskResubscriptionRequest`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskResubscriptionRequest\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"method\","]
#[doc = "    \"params\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"string\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"jsonrpc\": {"]
#[doc = "      \"title\": \"Jsonrpc\","]
#[doc = "      \"default\": \"2.0\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"2.0\""]
#[doc = "    },"]
#[doc = "    \"method\": {"]
#[doc = "      \"title\": \"Method\","]
#[doc = "      \"default\": \"tasks/resubscribe\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"tasks/resubscribe\""]
#[doc = "    },"]
#[doc = "    \"params\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskQueryParams\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskResubscriptionRequest {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub id: ::std::option::Option<Id>,
    #[serde(default = "defaults::task_resubscription_request_jsonrpc")]
    pub jsonrpc: ::std::string::String,
    pub method: ::std::string::String,
    pub params: TaskQueryParams,
}
impl ::std::convert::From<&TaskResubscriptionRequest> for TaskResubscriptionRequest {
    fn from(value: &TaskResubscriptionRequest) -> Self {
        value.clone()
    }
}
#[doc = "`TaskSendParams`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskSendParams\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"historyLength\": {"]
#[doc = "      \"title\": \"HistoryLength\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"$ref\": \"#/$defs/Message\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"pushNotification\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/PushNotificationConfig\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"title\": \"Sessionid\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskSendParams {
    #[serde(
        rename = "historyLength",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub history_length: ::std::option::Option<i64>,
    pub id: ::std::string::String,
    pub message: Message,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    #[serde(
        rename = "pushNotification",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub push_notification: ::std::option::Option<PushNotificationConfig>,
    #[serde(
        rename = "sessionId",
        default,
        skip_serializing_if = "::std::option::Option::is_none"
    )]
    pub session_id: ::std::option::Option<::std::string::String>,
}
impl ::std::convert::From<&TaskSendParams> for TaskSendParams {
    fn from(value: &TaskSendParams) -> Self {
        value.clone()
    }
}
#[doc = "An enumeration."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskState\","]
#[doc = "  \"description\": \"An enumeration.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"submitted\","]
#[doc = "    \"working\","]
#[doc = "    \"input-required\","]
#[doc = "    \"completed\","]
#[doc = "    \"canceled\","]
#[doc = "    \"failed\","]
#[doc = "    \"unknown\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TaskState {
    #[serde(rename = "submitted")]
    Submitted,
    #[serde(rename = "working")]
    Working,
    #[serde(rename = "input-required")]
    InputRequired,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "unknown")]
    Unknown,
}
impl ::std::convert::From<&Self> for TaskState {
    fn from(value: &TaskState) -> Self {
        value.clone()
    }
}
impl ::std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Submitted => write!(f, "submitted"),
            Self::Working => write!(f, "working"),
            Self::InputRequired => write!(f, "input-required"),
            Self::Completed => write!(f, "completed"),
            Self::Canceled => write!(f, "canceled"),
            Self::Failed => write!(f, "failed"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
impl ::std::str::FromStr for TaskState {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "submitted" => Ok(Self::Submitted),
            "working" => Ok(Self::Working),
            "input-required" => Ok(Self::InputRequired),
            "completed" => Ok(Self::Completed),
            "canceled" => Ok(Self::Canceled),
            "failed" => Ok(Self::Failed),
            "unknown" => Ok(Self::Unknown),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TaskState {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TaskState {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TaskState {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`TaskStatus`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskStatus\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"state\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"message\": {"]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/Message\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"state\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskState\""]
#[doc = "    },"]
#[doc = "    \"timestamp\": {"]
#[doc = "      \"title\": \"Timestamp\","]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"format\": \"date-time\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskStatus {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub message: ::std::option::Option<Message>,
    pub state: TaskState,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub timestamp: ::std::option::Option<::chrono::DateTime<::chrono::offset::Utc>>,
}
impl ::std::convert::From<&TaskStatus> for TaskStatus {
    fn from(value: &TaskStatus) -> Self {
        value.clone()
    }
}
#[doc = "`TaskStatusUpdateEvent`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TaskStatusUpdateEvent\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"id\","]
#[doc = "    \"status\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"final\": {"]
#[doc = "      \"title\": \"Final\","]
#[doc = "      \"default\": false,"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"id\": {"]
#[doc = "      \"title\": \"Id\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"status\": {"]
#[doc = "      \"$ref\": \"#/$defs/TaskStatus\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TaskStatusUpdateEvent {
    #[serde(rename = "final", default)]
    pub final_: bool,
    pub id: ::std::string::String,
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    pub status: TaskStatus,
}
impl ::std::convert::From<&TaskStatusUpdateEvent> for TaskStatusUpdateEvent {
    fn from(value: &TaskStatusUpdateEvent) -> Self {
        value.clone()
    }
}
#[doc = "`TextPart`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TextPart\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"text\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"metadata\": {"]
#[doc = "      \"title\": \"Metadata\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"anyOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"object\","]
#[doc = "          \"additionalProperties\": {}"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"text\": {"]
#[doc = "      \"title\": \"Text\","]
#[doc = "      \"type\": \"string\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"title\": \"Type\","]
#[doc = "      \"description\": \"Type of the part\","]
#[doc = "      \"default\": \"text\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"text\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"text\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct TextPart {
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub metadata:
        ::std::option::Option<::serde_json::Map<::std::string::String, ::serde_json::Value>>,
    pub text: ::std::string::String,
    #[doc = "Type of the part"]
    #[serde(rename = "type")]
    pub type_: ::std::string::String,
}
impl ::std::convert::From<&TextPart> for TextPart {
    fn from(value: &TextPart) -> Self {
        value.clone()
    }
}
#[doc = "`UnsupportedOperationError`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"UnsupportedOperationError\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"data\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"title\": \"Code\","]
#[doc = "      \"description\": \"Error code\","]
#[doc = "      \"default\": -32004,"]
#[doc = "      \"examples\": ["]
#[doc = "        -32004"]
#[doc = "      ],"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"const\": -32004"]
#[doc = "    },"]
#[doc = "    \"data\": {"]
#[doc = "      \"title\": \"Data\","]
#[doc = "      \"default\": null,"]
#[doc = "      \"const\": null"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"title\": \"Message\","]
#[doc = "      \"description\": \"A short description of the error\","]
#[doc = "      \"default\": \"This operation is not supported\","]
#[doc = "      \"examples\": ["]
#[doc = "        \"This operation is not supported\""]
#[doc = "      ],"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"const\": \"This operation is not supported\""]
#[doc = "    }"]
#[doc = "  }"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
pub struct UnsupportedOperationError {
    #[doc = "Error code"]
    pub code: i64,
    pub data: ::serde_json::Value,
    #[doc = "A short description of the error"]
    pub message: ::std::string::String,
}
impl ::std::convert::From<&UnsupportedOperationError> for UnsupportedOperationError {
    fn from(value: &UnsupportedOperationError) -> Self {
        value.clone()
    }
}
#[doc = r" Generation of default values for serde."]
pub mod defaults {
    pub fn agent_card_default_input_modes() -> ::std::vec::Vec<::std::string::String> {
        vec!["text".to_string()]
    }
    pub fn agent_card_default_output_modes() -> ::std::vec::Vec<::std::string::String> {
        vec!["text".to_string()]
    }
    pub fn cancel_task_request_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn cancel_task_response_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn get_task_push_notification_request_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn get_task_push_notification_response_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn get_task_request_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn get_task_response_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn jsonrpc_message_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn jsonrpc_request_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn jsonrpc_response_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn send_task_request_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn send_task_response_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn send_task_streaming_request_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn send_task_streaming_response_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn send_task_streaming_response_result() -> super::SendTaskStreamingResponseResult {
        super::SendTaskStreamingResponseResult::Variant2
    }
    pub fn set_task_push_notification_request_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn set_task_push_notification_response_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
    pub fn task_resubscription_request_jsonrpc() -> ::std::string::String {
        "2.0".to_string()
    }
}
