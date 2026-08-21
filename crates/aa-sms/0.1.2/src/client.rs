use crate::message::Message;
use std::error::Error;

#[derive(serde::Serialize)]
struct ApiCall<'a> {
    #[serde(flatten)]
    client: &'a Client,
    #[serde(flatten)]
    message: &'a Message,
}

/// An asynchronous `Client` to send a [`Message`] with.
///
/// To configure a `Client`, use [`Client::builder()`].
#[derive(serde::Serialize)]
pub struct Client {
    username: String,
    password: String,
    #[serde(skip_serializing)]
    reqwest_client: reqwest::Client,
}

impl Client {
    /// Creates a `ClientBuilder` to configure a `Client`.
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            ..ClientBuilder::default()
        }
    }

    /// Sends the `Message` using the configured `Client`.
    pub async fn send(self, message: Message) -> Result<(), Box<dyn Error>> {
        let api_call = ApiCall {
            client: &self,
            message: &message,
        };
        let res = match self
            .reqwest_client
            .post("https://sms.aa.net.uk/sms.cgi")
            .json(&api_call)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => { Err(Box::<dyn Error>::from(String::from("Invalid HTTP call."))) }?,
        };
        let body = match res.text().await {
            Ok(body) => body,
            Err(_) => {
                Err(Box::<dyn Error>::from(String::from(
                    "Invalid HTTP response.",
                )))
            }?,
        };

        let status_and_message = body.split_once(":");

        if status_and_message.is_none() {
            return Err(Box::<dyn Error>::from(String::from(
                "Invalid API response.",
            )));
        }

        let status = status_and_message.unwrap().0;
        let message = status_and_message.unwrap().1;

        if status == "ERR" {
            return Err(Box::<dyn Error>::from(message));
        }

        Ok(())
    }
}

/// A builder to construct the properties of a [`Client`].
#[derive(Default)]
pub struct ClientBuilder {
    username: Option<String>,
    password: Option<String>,
}

impl ClientBuilder {
    /// Returns a `Client` that uses this `ClientBuilder` configuration.
    pub fn build(self) -> Client {
        Client {
            username: self.username.expect("Cannot build sms without `username`."),
            password: self.password.expect("Cannot build sms without `password`."),
            reqwest_client: reqwest::Client::new(),
        }
    }

    /// Sets the `username` to be used to authenticate the `Client`.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the `password` to be used to authenticate the `Client`.
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_default_all_none() {
        let cb = Client::builder();
        assert_eq!(cb.username, None);
        assert_eq!(cb.password, None);
    }

    #[test]
    fn builder_setters_work() {
        let cb = Client::builder().username("123");
        assert_eq!(cb.username, Some(String::from("123")));
        assert_eq!(cb.password, None);

        let cb = Client::builder().password("123");
        assert_eq!(cb.username, None);
        assert_eq!(cb.password, Some(String::from("123")));
    }

    #[test]
    fn builder_works() {
        let client = Client::builder().username("123").password("456").build();
        assert_eq!(client.username, "123");
        assert_eq!(client.password, "456");
    }

    #[test]
    fn builder_order_doesnt_matter() {
        let client1 = Client::builder().username("123").password("456").build();
        let client2 = Client::builder().password("456").username("123").build();
        assert_eq!(client1.username, client2.username);
        assert_eq!(client1.password, client2.password);
    }

    #[test]
    #[should_panic]
    fn build_without_username_fails() {
        Client::builder().password("123").build();
    }

    #[test]
    #[should_panic]
    fn build_without_password_fails() {
        Client::builder().username("123").build();
    }
}
