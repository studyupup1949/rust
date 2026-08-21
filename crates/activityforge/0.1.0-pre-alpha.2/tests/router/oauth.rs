use activityforge::app::App;
use activityforge::app::oauth::{
    AuthorizationCode, AuthorizationRequest, ClientSecret, CodeChallenge, OAuthClient,
    OAuthClientRegister, OAuthClientResponse, OAuthToken, ReadScope, RefreshTokenRequest,
    ResponseType, Scope, ScopeList, TokenRequest, WriteScope,
};
use activityforge::crypto::{AlgorithmName, KeyType, PrivateKey, PublicKey};
use activityforge::db::{Db, Iri as DbIri, Name as DbName, Person as DbPerson, Uuid};
use activityforge::{Error, Result};
use activitystreams_vocabulary::MimeType;

use base64::Encoding;
use http::{StatusCode, header};
use oauth::endpoint::Scope as OAuthScope;
use sha2::Digest;

use crate::router::person::create_local_person;

crate::router_test! {
    register_oauth_client => run_register_oauth_client_test(db, app) {
        let http_client = reqwest::Client::new();

        let person_name = DbName::try_from("Test-OAuth-2.0-Register-Client")?;
        let (person, client_privkey, client_res) = register_oauth_client(
            app,
            db,
            &http_client,
            &person_name,
            "super-secret-password",
        ).await?;

        let oauth_client = OAuthClient::get(db, &client_res.client_id()).await?;

        assert_eq!(oauth_client.key_ids().len(), 1);

        let keys = oauth_client.keys(&db).await?;
        let pubkey: PublicKey = keys.get(0).unwrap().try_into()?;
        assert_eq!(pubkey, client_privkey.public_key());

        Ok(())
    }
}

crate::router_test! {
    authorize_oauth_client => run_authorize_client_test(db, app) {
        let app_uri = app.uri();
        let http_client = reqwest::Client::new();

        let person_name = DbName::try_from("OAuth-2.0-Test-Authorize-Client")?;
        let (person, _client_privkey, client_res) = register_oauth_client(
            app,
            db,
            &http_client,
            &person_name,
            "super-secret-password",
        ).await?;

        let client_id = client_res.client_id();
        let client_secret = client_res.client_secret();

        let oauth_token = create_oauth_token(
            app,
            &http_client,
            &client_id,
            client_secret,
            b"super-secret-pkce",
        ).await?;

        // check that the refresh token flow works

        let refresh_token = oauth_token
            .refresh_token()
            .ok_or(Error::http("oauth: missing refresh token"))?;

        let body = RefreshTokenRequest::new()
            .with_refresh_token(refresh_token)
            .with_scope([Scope::Read(ReadScope::Read), Scope::Write(WriteScope::Write)])
            .to_string();

        let res = http_client
            .post(format!("{app_uri}/oauth/refresh"))
            .header(header::CONTENT_TYPE, MimeType::ApplicationWwwFormUrlEncoded.as_str())
            .header(header::CONTENT_LENGTH, body.len())
            .basic_auth(client_id.to_string(), Some(client_secret))
            .body(body)
            .send()
            .await?;

        log::debug!("oauth: refresh token response: {res:?}");

        assert_eq!(res.status(), StatusCode::OK);

        let oauth_token = res.json::<OAuthToken>().await?;

        log::debug!("oauth: refresh token: {oauth_token}");

        Ok(())
    }
}

/// Registers an OAuth-2.0 client.
pub(crate) async fn register_oauth_client(
    app: &App,
    db: &Db,
    http_client: &reqwest::Client,
    name: &DbName,
    password: &str,
) -> Result<(DbPerson, PrivateKey, OAuthClientResponse)> {
    let person = create_local_person(db, app.uri(), &name, password).await?;

    let app_uri = app.uri();

    let res = http_client
        .post(format!("{app_uri}/oauth/authenticate"))
        .basic_auth(person.name().as_str(), Some(password))
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::OK);

    let client_privkey = PrivateKey::random(KeyType::Ed25519)?;
    let client_jwk: jwt::jwk::Jwk = client_privkey.public_key().try_into()?;

    let oauth_token = res.json::<OAuthToken>().await?;

    let oauth_scopes = ScopeList::from([
        Scope::Profile,
        Scope::Read(ReadScope::Read),
        Scope::Write(WriteScope::Write),
    ]);

    let oauth_register = OAuthClientRegister::new()
        .with_redirect_uris([DbIri::try_from(format!("{app_uri}/oauth/callback"))?])
        .with_scope(OAuthScope::try_from(oauth_scopes).unwrap())
        .with_jwks(jwt::jwk::JwkSet {
            keys: vec![client_jwk],
        });

    let key = app.state().signing_key(AlgorithmName::Ed25519).await?;
    let token = oauth_token.token();

    OAuthToken::verify_token(key.public_key().key(), token)?;

    let res = http_client
        .post(format!("{app_uri}/oauth/register"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(oauth_register.to_string())
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::CREATED);

    res.json::<OAuthClientResponse>()
        .await
        .map(|res| (person, client_privkey, res))
        .map_err(|err| Error::http(format!("oauth: error parsing registration response: {err}")))
}

/// Creates an OAuth-2.0 access token using the provided credentials.
pub(crate) async fn create_oauth_token(
    app: &App,
    http_client: &reqwest::Client,
    client_id: &Uuid,
    client_secret: &ClientSecret,
    pkce_input: &[u8],
) -> Result<OAuthToken> {
    let app_uri = app.uri();

    let pkce_digest = sha2::Sha256::digest(pkce_input);
    let pkce_verifier = base64::Base64UrlUnpadded::encode_string(&pkce_digest);
    let pkce = CodeChallenge::from_verifier(&pkce_verifier)?;

    let authz_req = AuthorizationRequest::new()
        .with_response_type(ResponseType::Code)
        .with_client_id(*client_id)
        .with_code_challenge(pkce);

    let res = http_client
        .get(format!("{app_uri}/oauth/authorize?{authz_req}"))
        .header(
            "Content-Type",
            MimeType::ApplicationWwwFormUrlEncoded.as_str(),
        )
        .basic_auth(client_id.to_string(), Some(client_secret))
        .send()
        .await?;

    log::debug!("oauth: authorization response: {res:?}");

    assert_eq!(res.status(), StatusCode::OK);

    let code = res
        .json::<AuthorizationCode>()
        .await
        .map_err(|err| Error::http(format!("{err}")))?;

    log::debug!("oauth: authorization code: {code}");

    let redirect_uri = app.state().oauth_callback_uri()?;

    let token_req = TokenRequest::new()
        .with_code(code.code())
        .with_redirect_uri(redirect_uri)
        .with_code_verifier(&pkce_verifier)
        .to_string();

    log::debug!("oauth: access code params: {token_req}");

    let res = http_client
        .post(format!("{app_uri}/oauth/token"))
        .header(
            "Content-Type",
            MimeType::ApplicationWwwFormUrlEncoded.as_str(),
        )
        .header("Content-Length", token_req.len())
        .basic_auth(client_id.to_string(), Some(client_secret))
        .body(token_req)
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::OK);

    log::debug!("oauth: token response: {res:?}");

    res.json::<OAuthToken>()
        .await
        .map_err(|err| Error::http(format!("oauth: invalid token response: {err}")))
}
