use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Wrapper used by proto JSON for repeated string values in maps.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StringList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Ordered string values.
    pub list: Vec<String>,
}

/// Security requirement mapping from scheme name to scopes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityRequirement {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    /// Required schemes and scope lists.
    pub schemes: BTreeMap<String, StringList>,
}

/// Supported security scheme variants.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SecurityScheme {
    #[serde(rename = "apiKeySecurityScheme")]
    /// API key security scheme.
    ApiKeySecurityScheme(ApiKeySecurityScheme),
    #[serde(rename = "httpAuthSecurityScheme")]
    /// HTTP auth security scheme.
    HttpAuthSecurityScheme(HttpAuthSecurityScheme),
    #[serde(rename = "oauth2SecurityScheme")]
    /// OAuth 2.0 security scheme.
    OAuth2SecurityScheme(OAuth2SecurityScheme),
    #[serde(rename = "openIdConnectSecurityScheme")]
    /// OpenID Connect discovery scheme.
    OpenIdConnectSecurityScheme(OpenIdConnectSecurityScheme),
    #[serde(rename = "mtlsSecurityScheme")]
    /// Mutual TLS security scheme.
    MutualTlsSecurityScheme(MutualTlsSecurityScheme),
}

impl<'de> Deserialize<'de> for SecurityScheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        deserialize_security_scheme(value).map_err(serde::de::Error::custom)
    }
}

/// API key security scheme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySecurityScheme {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional description for human readers.
    pub description: Option<String>,
    /// Location of the API key, such as `header` or `query`.
    pub location: String,
    /// Header or parameter name carrying the key.
    pub name: String,
}

/// HTTP auth security scheme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpAuthSecurityScheme {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional description for human readers.
    pub description: Option<String>,
    /// Authentication scheme, such as `basic` or `bearer`.
    pub scheme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional bearer token format hint.
    pub bearer_format: Option<String>,
}

/// OAuth 2.0 security scheme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2SecurityScheme {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional description for human readers.
    pub description: Option<String>,
    /// Supported OAuth flow.
    pub flows: OAuthFlows,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional metadata discovery URL.
    pub oauth2_metadata_url: Option<String>,
}

/// OpenID Connect security scheme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenIdConnectSecurityScheme {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional description for human readers.
    pub description: Option<String>,
    /// OpenID Connect discovery URL.
    pub open_id_connect_url: String,
}

/// Mutual TLS security scheme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutualTlsSecurityScheme {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional description for human readers.
    pub description: Option<String>,
}

/// Supported OAuth 2.0 flow variants.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OAuthFlows {
    /// Authorization code flow.
    AuthorizationCode(AuthorizationCodeOAuthFlow),
    /// Client credentials flow.
    ClientCredentials(ClientCredentialsOAuthFlow),
    /// Implicit flow.
    Implicit(ImplicitOAuthFlow),
    /// Resource owner password flow.
    Password(PasswordOAuthFlow),
    /// Device code flow.
    DeviceCode(DeviceCodeOAuthFlow),
}

impl<'de> Deserialize<'de> for OAuthFlows {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        deserialize_oauth_flows(value).map_err(serde::de::Error::custom)
    }
}

/// Authorization code flow settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationCodeOAuthFlow {
    /// Authorization endpoint URL.
    pub authorization_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional refresh endpoint URL.
    pub refresh_url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    /// OAuth scopes and their descriptions.
    pub scopes: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "crate::types::is_false")]
    /// Whether PKCE is required for this flow.
    pub pkce_required: bool,
}

/// Client credentials flow settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCredentialsOAuthFlow {
    /// Token endpoint URL.
    pub token_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional refresh endpoint URL.
    pub refresh_url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    /// OAuth scopes and their descriptions.
    pub scopes: BTreeMap<String, String>,
}

/// Implicit flow settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImplicitOAuthFlow {
    /// Authorization endpoint URL.
    pub authorization_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional refresh endpoint URL.
    pub refresh_url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    /// OAuth scopes and their descriptions.
    pub scopes: BTreeMap<String, String>,
}

/// Password flow settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordOAuthFlow {
    /// Token endpoint URL.
    pub token_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional refresh endpoint URL.
    pub refresh_url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    /// OAuth scopes and their descriptions.
    pub scopes: BTreeMap<String, String>,
}

/// Device code flow settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeOAuthFlow {
    /// Device authorization endpoint URL.
    pub device_authorization_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional refresh endpoint URL.
    pub refresh_url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    /// OAuth scopes and their descriptions.
    pub scopes: BTreeMap<String, String>,
}

fn deserialize_security_scheme(value: Value) -> Result<SecurityScheme, String> {
    let Value::Object(mut object) = value else {
        return Err("security scheme must be a JSON object".to_owned());
    };

    if object.len() == 1 {
        let (key, value) = object
            .into_iter()
            .next()
            .ok_or_else(|| "security scheme object cannot be empty".to_owned())?;
        return match key.as_str() {
            "apiKeySecurityScheme" => {
                deserialize_variant(value, SecurityScheme::ApiKeySecurityScheme)
            }
            "httpAuthSecurityScheme" => {
                deserialize_variant(value, SecurityScheme::HttpAuthSecurityScheme)
            }
            "oauth2SecurityScheme" => {
                deserialize_variant(value, SecurityScheme::OAuth2SecurityScheme)
            }
            "openIdConnectSecurityScheme" => {
                deserialize_variant(value, SecurityScheme::OpenIdConnectSecurityScheme)
            }
            "mtlsSecurityScheme" => {
                deserialize_variant(value, SecurityScheme::MutualTlsSecurityScheme)
            }
            _ => Err(format!("unknown security scheme variant: {key}")),
        };
    }

    let type_name = object
        .remove("type")
        .and_then(|value| match value {
            Value::String(value) => Some(value),
            _ => None,
        })
        .ok_or_else(|| "security scheme must contain either a proto oneof tag or a Python SDK 'type' discriminator".to_owned())?;

    match type_name.as_str() {
        "apiKey" => {
            if let Some(location) = object.remove("in") {
                object.insert("location".to_owned(), location);
            }
            deserialize_variant(Value::Object(object), SecurityScheme::ApiKeySecurityScheme)
        }
        "http" => deserialize_variant(
            Value::Object(object),
            SecurityScheme::HttpAuthSecurityScheme,
        ),
        "oauth2" => {
            deserialize_variant(Value::Object(object), SecurityScheme::OAuth2SecurityScheme)
        }
        "openIdConnect" => deserialize_variant(
            Value::Object(object),
            SecurityScheme::OpenIdConnectSecurityScheme,
        ),
        "mutualTLS" | "mutualTls" | "mtls" => deserialize_variant(
            Value::Object(object),
            SecurityScheme::MutualTlsSecurityScheme,
        ),
        other => Err(format!(
            "unsupported security scheme type discriminator: {other}"
        )),
    }
}

fn deserialize_oauth_flows(value: Value) -> Result<OAuthFlows, String> {
    let Value::Object(mut object) = value else {
        return Err("oauth flows must be a JSON object".to_owned());
    };

    let mut chosen: Option<(&'static str, Value)> = None;
    for key in [
        "authorizationCode",
        "clientCredentials",
        "implicit",
        "password",
        "deviceCode",
    ] {
        match object.remove(key) {
            Some(Value::Null) | None => {}
            Some(value) => {
                if chosen.is_some() {
                    return Err("oauth flows must contain exactly one flow variant".to_owned());
                }
                chosen = Some((key, value));
            }
        }
    }

    if !object.is_empty() {
        let mut keys = object.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        return Err(format!(
            "oauth flows contained unexpected keys: {}",
            keys.join(", ")
        ));
    }

    let Some((key, value)) = chosen else {
        return Err("oauth flows must contain exactly one flow variant".to_owned());
    };

    match key {
        "authorizationCode" => deserialize_variant(value, OAuthFlows::AuthorizationCode),
        "clientCredentials" => deserialize_variant(value, OAuthFlows::ClientCredentials),
        "implicit" => deserialize_variant(value, OAuthFlows::Implicit),
        "password" => deserialize_variant(value, OAuthFlows::Password),
        "deviceCode" => deserialize_variant(value, OAuthFlows::DeviceCode),
        _ => Err(format!("unsupported oauth flow variant: {key}")),
    }
}

fn deserialize_variant<T, U>(value: Value, constructor: impl FnOnce(T) -> U) -> Result<U, String>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(value)
        .map(constructor)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        ApiKeySecurityScheme, AuthorizationCodeOAuthFlow, HttpAuthSecurityScheme,
        OAuth2SecurityScheme, OAuthFlows, OpenIdConnectSecurityScheme, SecurityScheme,
    };

    #[test]
    fn security_scheme_serializes_as_externally_tagged_enum() {
        let scheme = SecurityScheme::ApiKeySecurityScheme(ApiKeySecurityScheme {
            description: None,
            location: "header".to_owned(),
            name: "X-API-Key".to_owned(),
        });

        let json = serde_json::to_string(&scheme).expect("scheme should serialize");
        assert_eq!(
            json,
            r#"{"apiKeySecurityScheme":{"location":"header","name":"X-API-Key"}}"#
        );
    }

    #[test]
    fn oauth_flows_serializes_with_variant_name() {
        let mut scopes = BTreeMap::new();
        scopes.insert("read".to_owned(), "Read access".to_owned());

        let scheme = OAuth2SecurityScheme {
            description: None,
            flows: OAuthFlows::AuthorizationCode(AuthorizationCodeOAuthFlow {
                authorization_url: "https://example.com/authorize".to_owned(),
                token_url: "https://example.com/token".to_owned(),
                refresh_url: None,
                scopes,
                pkce_required: true,
            }),
            oauth2_metadata_url: None,
        };

        let json = serde_json::to_string(&scheme).expect("oauth2 scheme should serialize");
        assert!(json.contains(
            r#""authorizationCode":{"authorizationUrl":"https://example.com/authorize""#
        ));
        assert!(json.contains(r#""pkceRequired":true"#));
    }

    #[test]
    fn security_scheme_deserializes_python_sdk_api_key_shape() {
        let json = serde_json::json!({
            "type": "apiKey",
            "description": "Header auth",
            "in": "header",
            "name": "X-API-Key"
        });

        let scheme: SecurityScheme =
            serde_json::from_value(json).expect("scheme should deserialize");

        match &scheme {
            SecurityScheme::ApiKeySecurityScheme(scheme) => {
                assert_eq!(scheme.location, "header");
                assert_eq!(scheme.name, "X-API-Key");
            }
            _ => panic!("expected api key scheme"),
        }

        let reserialized = serde_json::to_string(&scheme).expect("scheme should serialize");
        assert_eq!(
            reserialized,
            r#"{"apiKeySecurityScheme":{"description":"Header auth","location":"header","name":"X-API-Key"}}"#
        );
    }

    #[test]
    fn security_scheme_deserializes_python_sdk_http_shape() {
        let json = serde_json::json!({
            "type": "http",
            "scheme": "bearer",
            "bearerFormat": "JWT"
        });

        let scheme: SecurityScheme =
            serde_json::from_value(json).expect("scheme should deserialize");

        assert!(matches!(
            scheme,
            SecurityScheme::HttpAuthSecurityScheme(HttpAuthSecurityScheme { scheme, .. }) if scheme == "bearer"
        ));
    }

    #[test]
    fn security_scheme_deserializes_python_sdk_openid_shape() {
        let json = serde_json::json!({
            "type": "openIdConnect",
            "openIdConnectUrl": "https://example.com/.well-known/openid-configuration"
        });

        let scheme: SecurityScheme =
            serde_json::from_value(json).expect("scheme should deserialize");

        assert!(matches!(
            scheme,
            SecurityScheme::OpenIdConnectSecurityScheme(OpenIdConnectSecurityScheme { open_id_connect_url, .. })
                if open_id_connect_url == "https://example.com/.well-known/openid-configuration"
        ));
    }

    #[test]
    fn oauth_flows_deserialize_python_sdk_object_shape() {
        let json = serde_json::json!({
            "authorizationCode": {
                "authorizationUrl": "https://example.com/authorize",
                "tokenUrl": "https://example.com/token",
                "scopes": {
                    "read": "Read access"
                },
                "pkceRequired": true
            }
        });

        let flows: OAuthFlows = serde_json::from_value(json).expect("flows should deserialize");
        assert!(matches!(
            flows,
            OAuthFlows::AuthorizationCode(AuthorizationCodeOAuthFlow {
                pkce_required: true,
                ..
            })
        ));
    }

    #[test]
    fn security_scheme_deserializes_python_sdk_oauth2_shape() {
        let json = serde_json::json!({
            "type": "oauth2",
            "flows": {
                "authorizationCode": {
                    "authorizationUrl": "https://example.com/authorize",
                    "tokenUrl": "https://example.com/token",
                    "scopes": {
                        "read": "Read access"
                    }
                }
            }
        });

        let scheme: SecurityScheme =
            serde_json::from_value(json).expect("scheme should deserialize");

        assert!(matches!(
            scheme,
            SecurityScheme::OAuth2SecurityScheme(OAuth2SecurityScheme {
                flows: OAuthFlows::AuthorizationCode(_),
                ..
            })
        ));
    }

    #[test]
    fn security_scheme_deserializes_python_sdk_mutual_tls_shape() {
        let json = serde_json::json!({
            "type": "mutualTLS",
            "description": "mTLS client cert"
        });

        let scheme: SecurityScheme =
            serde_json::from_value(json).expect("scheme should deserialize");

        assert!(matches!(scheme, SecurityScheme::MutualTlsSecurityScheme(_)));
    }
}
