use std::str::FromStr;

use crate::{Authority, Endpoint, SocketAddress};

impl FromStr for Authority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(socket) = SocketAddress::from_str(s) {
            Ok(socket.to_authority())
        } else if let Ok(endpoint) = Endpoint::from_str(s) {
            Ok(endpoint.to_authority())
        } else {
            Err(())
        }
    }
}
