use std::convert::TryFrom;

use crate::{Authority, Domain, Endpoint, Host};

impl Domain {
    //! Conversions

    /// Converts the domain to an endpoint with the port.
    pub fn to_endpoint(self, port: u16) -> Endpoint {
        Endpoint::new(self, port)
    }

    /// Converts the domain to a host.
    pub fn to_host(self) -> Host {
        Host::Name(self)
    }

    /// Converts the domain to an authority with the port.
    pub fn to_authority(self, port: u16) -> Authority {
        Authority::new(self.to_host(), port)
    }
}

impl TryFrom<String> for Domain {
    type Error = ();

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if Domain::is_valid_name_str(name.as_str(), false) {
            Ok(unsafe { Domain::new(name) })
        } else {
            Err(())
        }
    }
}

impl From<Domain> for String {
    fn from(domain: Domain) -> Self {
        domain.export_name()
    }
}

impl TryFrom<&str> for Domain {
    type Error = ();

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if Domain::is_valid_name_str(name, false) {
            Ok(unsafe { Domain::new(name.to_string()) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<Vec<u8>> for Domain {
    type Error = ();

    fn try_from(name: Vec<u8>) -> Result<Self, Self::Error> {
        if Domain::is_valid_name(name.as_slice(), false) {
            Ok(unsafe { Domain::new(String::from_utf8_unchecked(name)) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&[u8]> for Domain {
    type Error = ();

    fn try_from(name: &[u8]) -> Result<Self, Self::Error> {
        if Domain::is_valid_name(name, false) {
            Ok(unsafe { Domain::new(std::str::from_utf8_unchecked(name).to_string()) })
        } else {
            Err(())
        }
    }
}
