use crate::app::oauth::{OAuthToken, OAuthTokenType};
use crate::{Error, Result};

use super::AppState;

impl AppState {
    /// Attempts to find an OAuth-2.0 token record by the provided JWT token + type.
    pub async fn find_oauth_token(
        &self,
        token: &str,
        token_type: OAuthTokenType,
    ) -> Result<Option<OAuthToken>> {
        let db = self.db().await;
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let token = OAuthToken::find_by_token_tx(&mut dbtx, token, token_type)
            .await
            .map_err(|err| Error::db(format!("app: state: {err}")))?;

        dbtx.commit()
            .await
            .map(|_| token)
            .map_err(|err| Error::db(format!("app: state: {err}")))
    }
}
