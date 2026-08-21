use crate::{Authority, Domain, Endpoint};

impl Endpoint {
    //! Conversions

    /// Exports the endpoint to an authority.
    pub fn to_authority(self) -> Authority {
        let (domain, port) = self.export();
        Authority::new(domain.to_host(), port)
    }
}

impl<D: Into<Domain>> From<(D, u16)> for Endpoint {
    fn from(t: (D, u16)) -> Self {
        Endpoint::new(t.0.into(), t.1)
    }
}

impl From<Endpoint> for (Domain, u16) {
    fn from(endpoint: Endpoint) -> Self {
        endpoint.export()
    }
}
