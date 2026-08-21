pub use content::Content;
pub use message::Message;

pub mod content;
pub mod message;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let result = Message::email_builder()
            .subject("The Email")
            .recipient("info@abineo.com")
            .body(Content::builder()
                .title("Hello, world!")
                .subtitle("Lorem ipsum dolor")
                .text("Now that we know who you are, I know who I am")
                .secret("42"))
            .build();

        assert!(result.is_ok());
    }
}
