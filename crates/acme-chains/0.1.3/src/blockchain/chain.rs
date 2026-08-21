/*
    Appellation: chain
    Context:
    Creator: FL03 <jo3mccain@icloud.com> (https://pzzld.eth.link/)
    Description:
        ... Summary ...
 */

use serde::{Deserialize, Serialize};

use crate::Block;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Chain<T> {
    pub application: T,
}