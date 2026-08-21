//! # AAP — Agent Accountability Protocol
//!
//! The accountability layer MCP and A2A don't have.
//!
//! ```
//! use aap_protocol::{KeyPair, Identity, Authorization, Level, Provenance, AuditChain, AuditResult};
//!
//! # fn main() -> aap_protocol::Result<()> {
//! let supervisor = KeyPair::generate();
//! let agent      = KeyPair::generate();
//!
//! let identity = Identity::new(
//!     "aap://acme/worker/bot@1.0.0",
//!     vec!["write:files".into()],
//!     &agent, &supervisor,
//!     "did:key:z6MkSupervisor",
//! )?;
//!
//! let auth = Authorization::new(
//!     &identity.id,
//!     Level::Supervised,
//!     vec!["write:files".into()],
//!     false, // not physical
//!     &supervisor,
//!     "did:key:z6MkSupervisor",
//! )?;
//!
//! assert!(auth.is_valid());
//! # Ok(())
//! # }
//! ```
//!
//! ## Physical World Rule
//!
//! ```
//! # use aap_protocol::{KeyPair, Authorization, Level, AAPError};
//! # let supervisor = KeyPair::generate();
//! let result = Authorization::new(
//!     "aap://factory/robot/arm@1.0.0",
//!     Level::Autonomous,   // Level 4
//!     vec!["move:arm".into()],
//!     true,                // physical = true
//!     &supervisor,
//!     "did:key:z6Mk",
//! );
//! assert!(matches!(result, Err(AAPError::PhysicalWorldViolation { .. })));
//! ```

mod crypto;
mod errors;
mod identity;
mod authorization;
mod provenance;
mod audit;

pub use crypto::{KeyPair, verify_signature, sha256_of};
pub use errors::{AAPError, Result};
pub use identity::Identity;
pub use authorization::{Authorization, Level};
pub use provenance::Provenance;
pub use audit::{AuditChain, AuditEntry, AuditResult};
