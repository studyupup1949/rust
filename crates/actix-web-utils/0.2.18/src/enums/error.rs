use std::{fmt::{self}, str::FromStr};

use crate::dtos::message::MessageResource;


/// This is supposed to be used whenever you have an error in your code and want to be more specific about it. 
/// Fits in with most CRUD web apps. What you send back to the client is a MessageResource, not the error itself!
#[derive(Debug, Clone)]
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
impl FromStr for Error {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let error_name_option = string.get(14..24);
        let error_name_whole = match error_name_option {
            Some(error_name_whole) => error_name_whole,
            None => return Err(Error::Unspecified),
        };
        if error_name_whole.starts_with("Unspecified") {
            return Err(Self::Unspecified)
        } else if error_name_whole.starts_with("UnexpectedStatusCode") {
            let expected_str_index = string.find("Expected: ").unwrap() + 10;
            let actual_str_index = string.find("Actual: ").unwrap() + 8;
            let expected_status_code = string.get(expected_str_index..expected_str_index+3).unwrap();
            let actual_status_code = string.get(actual_str_index..actual_str_index+3).unwrap();
            
            let message_resources_string = string.get(string.find("receivedMessageResources").unwrap() + 26..string.len() - 1).unwrap();
            let message_resources: Vec<MessageResource> = serde_json::from_str(message_resources_string).unwrap();
            return Err(Self::UnexpectedStatusCode(expected_status_code.parse().unwrap(), actual_status_code.parse().unwrap(), message_resources));
        } else if error_name_whole.starts_with("Client") {

        } else if error_name_whole.starts_with("Network") {

        } else if error_name_whole.starts_with("Serialization") {

        } else if error_name_whole.starts_with("Database") {

        } else if error_name_whole.starts_with("Compute") {

        }

        Ok(Error::ClientError(MessageResource::new_from_str("msg")))
    }
}
impl std::error::Error for Error {}