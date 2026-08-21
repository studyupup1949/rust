use std::fmt::{self};

use crate::dtos::message::MessageResource;


/// This is supposed to be used whenever you have an error in your code and want to be more specific about it. 
/// Fits in with most CRUD web apps. What you send back to the client is a MessageResource, not the error itself!
#[derive(Debug)]
pub enum Error{
    /// Takes a Message and the query
    DatabaseError(MessageResource, String),
    /// Same as UnexpectedStatusCode but without the extra details.
    ClientError(MessageResource),
    /// Takes the status code you expected, the actual status code, and the ErrorMessage. This is meant to be used when your app tries to use an API, be it internal or external.
    UnexpectedStatusCode(u16, u16, Vec<MessageResource>),
    /// Try and never use this error, unless you really need to.
    Unspecified,
    /// If you had an error serializing/deserializing and wish to display more details. Such as the entire Json as a string, this is how.
    SerdeError(MessageResource, String),
    /// Normally used in compute heavy operations, such as Hashing.
    ComputeError(MessageResource),
    /// Self explanatory, Network related error.
    NetworkError(MessageResource)
    
}
impl fmt::Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *&self {
            Error::Unspecified => write!(f, "Error of type Unspecified."),
            Error::NetworkError(message) => write!(f, "Error of type Network.\nMessageResource: {}", message),
            Error::UnexpectedStatusCode(expected, actual, messages) => write!(f, "Error of type UnexpectedStatusCode.\nExpected: {}\nActual: {}\nreceivedMessageResources: {:?}", expected, actual, messages),
            Error::ClientError(message) => write!(f, "Error of type Client.\nMessageResource: {}", message),
            Error::SerdeError(message, recieved) => write!(f, "Error of type Serialization/Deserialization.\nMessageResource: {:?}, Object attempted to be serded: {}", message, recieved),
            Error::DatabaseError(message, query) => write!(f, "Error of type Database.\nMessageResource: {}, \nQuery: {}", message, query),
            Error::ComputeError(message) => write!(f, "Error of type Compute.\nMessageResource: {}", message),
        }
    }
}
impl std::error::Error for Error {}