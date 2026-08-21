mod link;
mod mention;
mod mime;
mod url;

use isolang::Language;

pub use self::mime::Mime;
pub use self::url::Url;
pub use link::Link;
pub use mention::Mention;

pub trait LinkType {
    fn href(&self) -> &Url;
    fn media_type(&self) -> Option<&Mime> {
        None
    }
    fn name(&self) -> Option<&String> {
        None
    }
    fn rel(&self) -> Option<&String> {
        None
    }
    fn href_lang(&self) -> Option<&Language> {
        None
    }
}
