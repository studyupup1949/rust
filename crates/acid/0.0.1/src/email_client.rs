use {crate::domain::UserEmail, reqwest::Client, serde_json::json, std::collections::HashMap};

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: UserEmail,
    api_key: String,
    api_token: String,
}

impl EmailClient {
    pub fn new(base_url: String, sender: UserEmail, api_key: String, api_token: String) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            http_client,
            base_url,
            sender,
            api_key,
            api_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &UserEmail,
        subject: &str,
        template_id: &u64,
        variables: HashMap<String, String>,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/send", self.base_url);

        let request_body = json!({
            "Messages":[
                {
                    "From": {
                        "Email": self.sender.as_ref(),
                        "Name": self.sender.as_ref()
                    },
                    "To": [
                        {
                            "Email": recipient.as_ref()
                        }
                    ],
                    "Subject": subject,
                    "TemplateID": template_id,
                    "TemplateLanguage": true,
                    "Variables": variables
                }
            ]
        });

        self.http_client
            .post(&url)
            .basic_auth(&self.api_key, Some(&self.api_token))
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::domain::UserEmail,
        crate::email_client::EmailClient,
        claims::{assert_err, assert_ok},
        fake::faker::internet::en::SafeEmail,
        fake::faker::lorem::en::Sentence,
        fake::{Fake, Faker},
        serde_json::json,
        std::collections::HashMap,
        wiremock::matchers::any,
        wiremock::matchers::{header, header_exists, method, path},
        wiremock::Request,
        wiremock::{Mock, MockServer, ResponseTemplate},
    };

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                let data = json!(body);
                let message = &data["Messages"][0];

                message["From"].get("Email").is_some()
                    && message["From"].get("Name").is_some()
                    && message["To"][0].get("Email").is_some()
                    && message.get("Subject").is_some()
                    && message.get("TemplateID").is_some()
                    && message.get("Variables").is_some()
            } else {
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn template_id() -> u64 {
        (8..20).fake()
    }

    fn variables() -> HashMap<String, String> {
        let mut variables = HashMap::new();
        variables.insert(Faker.fake::<String>(), Faker.fake::<String>());
        variables.insert(Faker.fake::<String>(), Faker.fake::<String>());

        variables
    }

    fn email() -> UserEmail {
        UserEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(base_url, email(), Faker.fake(), Faker.fake())
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/send"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(&email(), &subject(), &template_id(), variables())
            .await;

        // Assert
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = UserEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Faker.fake(), Faker.fake());
        let subscriber_email = UserEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let template_id: u64 = (8..20).fake();

        let mut variables = HashMap::new();
        variables.insert(Faker.fake::<String>(), Faker.fake::<String>());
        variables.insert(Faker.fake::<String>(), Faker.fake::<String>());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&subscriber_email, &subject, &template_id, variables)
            .await;

        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = UserEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Faker.fake(), Faker.fake());
        let subscriber_email = UserEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let template_id: u64 = (8..20).fake();

        let mut variables = HashMap::new();
        variables.insert(Faker.fake::<String>(), Faker.fake::<String>());
        variables.insert(Faker.fake::<String>(), Faker.fake::<String>());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&subscriber_email, &subject, &template_id, variables)
            .await;

        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = UserEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Faker.fake(), Faker.fake());
        let subscriber_email = UserEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let template_id: u64 = (8..20).fake();

        let mut variables = HashMap::new();
        variables.insert(Faker.fake::<String>(), Faker.fake::<String>());
        variables.insert(Faker.fake::<String>(), Faker.fake::<String>());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&subscriber_email, &subject, &template_id, variables)
            .await;

        // Assert
        assert_err!(outcome);
    }
}
