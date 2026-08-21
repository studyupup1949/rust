use std::time::Duration;

use chrono::Utc;

use activityforge::app::oauth::{OAuthGrant, OAuthToken, OAuthTokenType, Scope};
use activityforge::db::Iri;

crate::db_test! {
    oauth_grant => run_tests(db) {
        let host = Iri::try_from("https://example.dev")?;

        let person_uuid = db.rand_uuid();
        let owner_id = Iri::try_from(format!("{host}/persons/{person_uuid}"))?;
        let client_id = db.rand_uuid();
        let scopes = [Scope::Profile];
        let redirect_uri = Iri::try_from(format!("{host}/oauth/authorize"))?;
        let until = Utc::now() + Duration::from_hours(1);

        let mut oauth_grant = OAuthGrant::new()
            .with_owner_id(owner_id)
            .with_client_id(client_id)
            .with_redirect_uri(redirect_uri)
            .with_until(until)
            .with_scopes(scopes)?;

        let grant_id = oauth_grant.insert(&db).await?;

        // NOTE: the token fields would be JWT tokens in real usage.
        let mut oauth_token = OAuthToken::new()
            .with_token("fake_test_token")
            .with_refresh_token("fake_test_refresh_token")
            .with_until(until)
            .with_token_type(OAuthTokenType::Access)
            .with_grant_id(grant_id);

        let token_id = oauth_token.insert(&db).await?;

        let get_grant = OAuthGrant::get(&db, &grant_id).await?;

        let fetch_grant = OAuthGrant::find_by_token(
            &db,
            oauth_token.token(),
            OAuthTokenType::Access,
        ).await?;

        let refresh_grant = OAuthGrant::find_by_token(
            &db,
            oauth_token.refresh_token().unwrap(),
            OAuthTokenType::Refresh,
        ).await?;

        assert_eq!(fetch_grant.as_ref(), Some(&get_grant));
        assert_eq!(refresh_grant.as_ref(), Some(&get_grant));

        let get_token = OAuthToken::get(&db, &token_id).await?;

        let fetch_token = OAuthToken::find_by_token(
            &db,
            &oauth_token.token(),
            OAuthTokenType::Access,
        ).await?;

        let refresh_token = OAuthToken::find_by_token(
            &db,
            &oauth_token.refresh_token().unwrap(),
            OAuthTokenType::Refresh,
        ).await?;

        assert_eq!(fetch_token.as_ref(), Some(&get_token));
        assert_eq!(refresh_token.as_ref(), Some(&get_token));

        oauth_grant.delete(&db).await?;

        // Check that when a grant is deleted, the associated token is too

        assert!(OAuthToken::get(&db, &token_id).await.is_err());

        assert!(OAuthToken::find_by_token(
                &db,
                oauth_token.token(),
                OAuthTokenType::Access,
        ).await?.is_none());

        assert!(OAuthToken::find_by_token(
                &db,
                oauth_token.refresh_token().unwrap(),
                OAuthTokenType::Refresh,
        ).await?.is_none());

        Ok(())
    }
}
