use crate::Host;

/// Represents a host with an associated port.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Authority {
    host: Host,
    port: u16,
}

impl Authority {
    //! Constructors

    /// Creates a new authority.
    pub const fn new(host: Host, port: u16) -> Self {
        Self { host, port }
    }
}

impl Authority {
    //! Properties

    /// Gets the host.
    pub const fn host(&self) -> &Host {
        &self.host
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}

impl Authority {
    //! Deconstructors

    /// Exports the authority as a tuple.
    pub fn export(self) -> (Host, u16) {
        (self.host, self.port)
    }

    /// Exports the host.
    pub fn export_host(self) -> Host {
        self.host
    }
}
