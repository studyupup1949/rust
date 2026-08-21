use oauth::endpoint::{Scope as OAuthScope, Scopes, WebRequest};
use serde::{Deserialize, Serialize};

use activitystreams_vocabulary::{impl_default, impl_display};

use super::{AdminScope, ReadScope, Scope, WriteScope};
use crate::{Error, Result};

/// Represents the default set of OAuth-2.0 endpoint scopes needed to access a resource.
///
/// A client must satisfy at least one of the scopes in the list.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct EndpointScopes {
    scopes: Vec<OAuthScope>,
}

impl EndpointScopes {
    /// Creates a new [EndpointScopes].
    pub fn new() -> Self {
        let scopes = [
            Scope::Profile,
            Scope::Push,
            Scope::Visit,
            Scope::Report,
            Scope::Triage,
            Scope::Maintain,
            Scope::Read(ReadScope::Read),
            Scope::Write(WriteScope::Write),
            Scope::Admin(AdminScope::Admin),
        ]
        .into_iter()
        .map(|s| OAuthScope::try_from(s).map_err(|err| Error::http(format!("oauth: scope: {err}"))))
        .collect::<Result<Vec<_>>>()
        .unwrap_or_default();

        Self { scopes }
    }

    /// Adds a scope to the list.
    pub fn add_scope(&mut self, scope: Scope) -> Result<()> {
        let scope: OAuthScope = scope
            .try_into()
            .map_err(|err| Error::http(format!("oauth: scope: error parsing scope: {err}")))?;

        if self.scopes.contains(&scope) {
            Err(Error::http(format!(
                "oauth: scope: duplicate endpoint scope: {scope}"
            )))
        } else {
            self.scopes.push(scope);
            Ok(())
        }
    }
}

impl_default!(EndpointScopes);
impl_display!(EndpointScopes, json);

impl<Request: WebRequest> Scopes<Request> for EndpointScopes {
    fn scopes(&mut self, _: &mut Request) -> &[OAuthScope] {
        self.scopes.as_ref()
    }
}
