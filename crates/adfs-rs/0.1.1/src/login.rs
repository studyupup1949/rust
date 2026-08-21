use aws_sdk_sts::{config::Credentials, types::Credentials as CredentialsType};
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::Client;
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::fmt::Display;
use std::{collections::HashMap, fs::File, io::Write};

#[derive(Debug, Deserialize)]
struct Attribute {
    #[serde(rename = "@Name")]
    name: String,

    #[serde(rename = "AttributeValue")]
    attribute_value: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AttributeStatement {
    #[serde(rename = "Attribute")]
    attributes: Vec<Attribute>,
}

#[derive(Debug, Deserialize)]
struct Assertion {
    #[serde(rename = "AttributeStatement")]
    attribute_statement: AttributeStatement,
}

#[derive(Debug, Deserialize)]
struct Response {
    #[serde(rename = "Assertion")]
    assertion: Assertion,
}

#[derive(Debug, Deserialize)]
struct AdLoginInput {
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct AdLoginForm {
    input: AdLoginInput,
}

#[derive(Debug, Deserialize)]
struct AdLoginBody {
    form: AdLoginForm,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "html")]
struct AdLoginHtml {
    body: AdLoginBody,
}

#[derive(Debug)]
struct Principal {
    principal_arn: String,
    role_arn: String,
}

#[derive(Debug, Deserialize)]
struct TargetCredentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
}

impl TargetCredentials {
    fn from_aws_creds(aws_creds: &CredentialsType) -> Self {
        return Self {
            access_key_id: aws_creds.access_key_id().to_string(),
            secret_access_key: aws_creds.secret_access_key().to_string(),
            session_token: aws_creds.session_token().to_string(),
        };
    }
}

impl Display for TargetCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "export AWS_ACCESS_KEY_ID={0}\nexport AWS_SECRET_ACCESS_KEY={1}\nexport AWS_SESSION_TOKEN={2}",
            self.access_key_id, self.secret_access_key, self.session_token
        );
    }
}

#[derive(Debug, Deserialize)]
struct AdfsConfig {
    target_credentials: TargetCredentials,
    account_id: Option<String>,
}

impl Display for AdfsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let account_id_export;
        if let Some(account_id) = &self.account_id {
            account_id_export = format!("export AWS_ACCOUNT_ID={account_id}");
        } else {
            account_id_export = "".to_string();
        }
        return write!(f, "{}\n{account_id_export}", self.target_credentials);
    }
}

async fn ad_login(
    client: &Client,
    ad_url: &str,
    username: &str,
    password: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut transdev_login_form = HashMap::new();
    transdev_login_form.insert("UserName", username);
    transdev_login_form.insert("Password", password);
    transdev_login_form.insert("AuthMethod", "FormsAuthentication");

    return client
        .post(format!(
            "https://{}/adfs/ls/IdpInitiatedSignOn.aspx?loginToRp=urn:amazon:webservices",
            ad_url
        ))
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 6.1; WOW64; Trident/7.0; rv:11.0) like Gecko",
        )
        .form(&transdev_login_form)
        .send()
        .await;
}

pub async fn login_command(
    ad_url: String,
    temp_creds_file: String,
    username: String,
    password: String,
    target_role: String,
    ad_role: String,
    role_session_name: String,
) {
    // login
    let client_builder = Client::builder().cookie_store(true);
    let client = client_builder.build().unwrap();

    let response = match ad_login(&client, &ad_url, &username, &password).await {
        Ok(response) => match response.error_for_status() {
            Ok(response) => response.text().await.unwrap(),
            Err(_) => ad_login(&client, &ad_url, &username, &password)
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        },
        Err(e) => panic!("Failed to login to AD {}", e),
    };

    let ad_login_html: AdLoginHtml = from_str(&response).unwrap();
    let saml_token = ad_login_html.body.form.input.value;

    let saml_token_decoded = general_purpose::STANDARD.decode(&saml_token).unwrap();
    let saml_token_decoded_str = str::from_utf8(&saml_token_decoded).unwrap();

    let saml_token_xml: Response = from_str(&saml_token_decoded_str).unwrap();
    let attributes = saml_token_xml.assertion.attribute_statement.attributes;
    let roles: &Vec<String> = &attributes
        .iter()
        .find(|attr| attr.name.ends_with("/Role"))
        .unwrap()
        .attribute_value;

    let role: Principal = match roles
        .iter()
        .map(|principal| principal.split(",").collect())
        .map(|principal_splitted: Vec<&str>| Principal {
            principal_arn: principal_splitted.first().unwrap().to_string(),
            role_arn: principal_splitted.last().unwrap().to_string(),
        })
        .find(|principal| principal.role_arn.ends_with(&ad_role))
    {
        Some(role) => role,
        None => panic!(
            "Specified AD ROLE does not exists or you don't have access [role: {}]",
            ad_role
        ),
    };

    let config = aws_config::from_env().region("eu-west-1").load().await;
    let sts_client = aws_sdk_sts::Client::new(&config);
    let assumed_role = sts_client
        .assume_role_with_saml()
        .role_arn(&role.role_arn)
        .principal_arn(&role.principal_arn)
        .saml_assertion(saml_token)
        .send()
        .await
        .unwrap();

    let assumed_creds = assumed_role.credentials.unwrap();
    let creds = Credentials::from_keys(
        assumed_creds.access_key_id,
        assumed_creds.secret_access_key,
        Some(assumed_creds.session_token),
    );

    let ad_config = aws_config::from_env()
        .credentials_provider(creds)
        .load()
        .await;
    let ad_sts_client = aws_sdk_sts::Client::new(&ad_config);

    let target_role = ad_sts_client
        .assume_role()
        .role_arn(target_role)
        .role_session_name(&role_session_name)
        .send()
        .await
        .unwrap();
    let target_creds = target_role.credentials.unwrap();
    let user_details = ad_sts_client.get_caller_identity().send().await.unwrap();

    let adfs_config = AdfsConfig {
        target_credentials: TargetCredentials::from_aws_creds(&target_creds),
        account_id: user_details.account,
    };
    let adfs_config_serialized = adfs_config.to_string();
    let mut adfs_config_file = File::create(temp_creds_file).unwrap();
    adfs_config_file
        .write(&adfs_config_serialized.as_bytes())
        .unwrap();
}
