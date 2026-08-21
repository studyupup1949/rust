use activitystreams_vocabulary::create_object;

use crate::{Error, Result};

mod openssh;

pub use openssh::{OpenSshKeyType, OpenSshPublicKey};

create_object! {
    /// A public SSH key.
    SshPublicKey: crate::ObjectType::SshPublicKey {}
}

impl SshPublicKey {
    /// Gets the OpenSSH public key.
    pub fn public_key(&self) -> Result<OpenSshPublicKey> {
        self.content
            .as_deref()
            .ok_or(Error::object("ssh_key: missing key"))
            .and_then(OpenSshPublicKey::try_from)
    }

    /// Sets the OpenSSH public key.
    pub fn set_public_key<I: Into<OpenSshPublicKey>>(&mut self, key: I) {
        self.content = Some(key.into().to_string());
    }

    /// Builder function that sets the OpenSSH public key.
    pub fn with_public_key<I: Into<OpenSshPublicKey>>(self, key: I) -> Self {
        Self {
            content: Some(key.into().to_string()),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{DateTime, Iri, Name};

    #[test]
    fn test_ssh_public_key() {
        let name = Name::try_from("somekey").unwrap();
        let attributed_to = Iri::try_from("https://example.dev/alice").unwrap();
        let published = "2026-07-10T13:37:42Z";
        let key = OpenSshPublicKey::try_from("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJQ2i70G8zuqjrEaIyxWQRvuivLjqELGDp71JBEHHhyd somekey@example.dev").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "SshPublicKey",
  "name": "{name}",
  "attributedTo": "{attributed_to}",
  "content": "{key}",
  "published": "{published}"
}}"#
        );

        let context_property = context::forgefed_context();

        let ssh_key = SshPublicKey::new()
            .with_context_property(context_property)
            .with_name(name)
            .with_attributed_to(attributed_to)
            .with_published(published.parse::<DateTime>().unwrap())
            .with_content(key);

        assert_eq!(serde_json::to_string_pretty(&ssh_key).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<SshPublicKey>(json_str.as_str()).unwrap(),
            ssh_key
        );
    }
}
