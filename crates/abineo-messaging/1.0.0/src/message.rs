use std::str::FromStr;

use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Content;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Message {
    Email {
        recipients: Vec<String>,
        subject: String,
        body: Vec<Content>,
    }
}

impl Message {
    pub fn email(recipients: Vec<String>, subject: String, body: Vec<Content>) -> Self {
        Message::Email { recipients, subject, body }
    }

    pub fn email_builder() -> EmailBuilder {
        EmailBuilder::default()
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct EmailBuilder {
    recipients: Vec<String>,
    subject: Option<String>,
    body: Vec<Content>,
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum EmailBuilderError {
    #[error("missing subject")]
    MissingSubject,
    #[error("{0}")]
    InvalidEmail(String),
}

impl EmailBuilder {
    pub fn recipient<T: Into<String>>(mut self, email: T) -> Self {
        self.recipients.push(email.into());
        self
    }

    pub fn subject<T: Into<String>>(mut self, subject: T) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn body<T: Into<Vec<Content>>>(mut self, body: T) -> Self {
        self.body = body.into();
        self
    }

    pub fn build(self) -> Result<Message, EmailBuilderError> {
        let subject = match self.subject {
            Some(s) => s,
            None => { return Err(EmailBuilderError::MissingSubject); }
        };

        for email in self.recipients.iter() {
            if let Err(error) = EmailAddress::from_str(email.as_str()) {
                return Err(EmailBuilderError::InvalidEmail(error.to_string()));
            }
        }

        Ok(Message::Email {
            recipients: self.recipients,
            subject,
            body: self.body,
        })
    }
}
