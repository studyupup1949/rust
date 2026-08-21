#![allow(deprecated)]

use activityforge::app::App;
use activityforge::app::oauth::{OAuthToken, OAuthTokenType};
use activityforge::crypto::{AlgorithmName, HttpPrivateKey, KeyType, PrivateKey};
use activityforge::db::{
    Actor, Factory as DbFactory, Inbox as DbInbox, Iri as DbIri, Key as DbKey, Name as DbName,
    Outbox as DbOutbox, Person as DbPerson, Repository as DbRepository, TableType,
};
use activityforge::{Activity, Error, Like, Repository, Result, Role, util};
use activitystreams_vocabulary::{ActivityType, Create, Iri, Item, MimeType, Multikey, Name};

use http::{Method, StatusCode, header};

use crate::router::factory::create_factory;
use crate::router::get_client_mailbox;
use crate::router::oauth::{create_oauth_token, register_oauth_client};
use crate::router::person::create_remote_person;

use super::{ED25519_KEY_ID, ED25519_PRIVKEY_BYTES};

crate::router_test! {
    get_repository => run_get_repository_test(db, app) {
        let factory_name = Name::try_from("test_get_repository_factory")?;
        let owner_name = DbName::try_from(format!("{factory_name}-owner"))?;

        let http_client = reqwest::Client::new();

        let (person, _client_privkey, client_res) = register_oauth_client(
            app,
            db,
            &http_client,
            &owner_name,
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

        let (db_factory, _key) = create_factory(
            app,
            &http_client,
            &oauth_token,
            &person,
            factory_name,
        ).await?;

        let repository_name = Name::try_from("test_get_repository")?;

        let key = HttpPrivateKey::from_bytes(
            ED25519_KEY_ID,
            AlgorithmName::Ed25519,
            &ED25519_PRIVKEY_BYTES,
        )
        .and_then(DbKey::try_from)?;

        let (db_repo, _create) = create_repository(
            &app,
            &http_client,
            &oauth_token,
            &person,
            &db_factory,
            repository_name,
        ).await?;

        let out_repo = get_repository(&http_client, &oauth_token, db_repo.id()).await?;
        let repo = db_repo.try_into_vocab(&*app.state().db().await).await?;

        assert_eq!(out_repo, repo, "[{out_repo}, {repo}]");

        Ok(())
    }
}

crate::router_test! {
    repository_client_like => run_repository_client_like_test(db, app) {
        let factory_name = Name::try_from("test_person_like_factory_outbox")?;
        let owner_name = DbName::try_from(format!("{factory_name}-owner"))?;

        let http_client = reqwest::Client::new();

        let (person, _client_privkey, client_res) = register_oauth_client(
            app,
            db,
            &http_client,
            &owner_name,
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

        let (db_factory, _key) = create_factory(
            app,
            &http_client,
            &oauth_token,
            &person,
            factory_name,
        ).await?;

        let repo_name = Name::try_from("test_person_like_repository_outbox")?;

        let (db_repo, _create) = create_repository(
            &app,
            &http_client,
            &oauth_token,
            &person,
            &db_factory,
            repo_name,
        ).await?;

        let repo_outbox = DbOutbox::get(&*app.state().db().await, &db_repo.outbox()).await?;
        let repo_outbox_id = repo_outbox.id();
        let person_id = person.id();

        let like_activity = add_client_repository_like(
            &http_client,
            &oauth_token,
            repo_outbox_id,
            &person_id,
            db_repo.id(),
        ).await?;

        let like = like_activity.as_like()?;

        let activities = get_client_mailbox(&http_client, &oauth_token, repo_outbox_id).await?;

        assert_eq!(activities.total_items(), Some(1u64));

        let in_activity: Activity = activities
            .items()
            .unwrap()
            .as_list()
            .unwrap()
            .iter()
            .find(|i| i.contains(ActivityType::Like))
            .unwrap()
            .try_into()?;

        let in_like = in_activity.as_like()?;

        assert_eq!(
            in_like.actor().unwrap(),
            like.actor().unwrap(),
            "[{in_like}, {like}]"
        );
        assert_eq!(
            in_like.object().unwrap(),
            like.object().unwrap(),
            "[{in_like}, {like}]"
        );

        Ok(())
    }
}

crate::router_test! {
    repository_server_like => run_repository_server_like_test(db, app) {
        let remote_id = DbIri::try_from("https://example.dev/test-person-like-repository-inbox")?;
        let remote_name = DbName::try_from("test-remote-person-like-repository-inbox")?;
        let (_, remote_person, remote_key) = create_remote_person(&app, &remote_id, &remote_name).await?;

        let factory_name = Name::try_from("test_person_like_factory_inbox")?;
        let owner_name = DbName::try_from(format!("{factory_name}-owner"))?;

        let http_client = reqwest::Client::new();

        let (person, _client_privkey, client_res) = register_oauth_client(
            app,
            db,
            &http_client,
            &owner_name,
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

        let (db_factory, _key) = create_factory(
            app,
            &http_client,
            &oauth_token,
            &person,
            factory_name,
        ).await?;

        let repo_name = Name::try_from("test_person_like_repository_inbox")?;

        let (db_repo, _create) = create_repository(
            &app,
            &http_client,
            &oauth_token,
            &person,
            &db_factory,
            repo_name,
        ).await?;

        let repo_inbox = DbInbox::get(&*app.state().db().await, &db_repo.inbox()).await?;
        let repo_inbox_id = repo_inbox.id();

        app.state()
            .create_grant(&Actor::person(remote_person), &[Role::Visit], db_repo.table_entry())
            .await?;

        let like_activity = add_server_repository_like(
            &remote_key,
            repo_inbox_id,
            &remote_id,
            db_repo.id(),
        ).await?;
        let like = like_activity.as_like()?;

        let activities = get_client_mailbox(&http_client, &oauth_token, repo_inbox_id).await?;

        assert_eq!(activities.total_items(), Some(1u64));

        let out_activity: Activity = activities
            .items()
            .unwrap()
            .as_list()
            .unwrap()
            .iter()
            .find(|i| i.contains(ActivityType::Like))
            .unwrap()
            .try_into()?;

        let out_like = out_activity.as_like()?;

        assert_eq!(
            out_like.actor().unwrap(),
            like.actor().unwrap(),
            "[{out_like}, {like}]"
        );
        assert_eq!(
            out_like.object().unwrap(),
            like.object().unwrap(),
            "[{out_like}, {like}]"
        );

        Ok(())
    }
}

pub(crate) async fn create_repository_request(
    app: &App,
    person: &DbPerson,
    repository_name: Name,
) -> Result<Create> {
    let repository_key_uuid = util::rand_uuid();
    let repository_key_id = TableType::Key.id_from_uuid(&app.uri(), repository_key_uuid)?;
    let repository_key = PrivateKey::random(KeyType::Ed25519)
        .and_then(DbKey::try_from)
        .map(|k| k.with_uuid(repository_key_uuid).with_id(repository_key_id))?;

    let repository_multikey = Multikey::try_from(&repository_key)?;

    let clone_uri = Iri::try_from(format!("{}/{}/{repository_name}", app.uri(), person.name()))?;
    let push_uri = Iri::try_from(format!("{}/{}/{repository_name}", app.uri(), person.name()))?;

    let repository = Repository::new()
        .with_name(repository_name)
        .with_assertion_method(repository_multikey)
        .with_clone_uri([clone_uri])
        .with_push_uri([push_uri]);

    Ok(Create::new()
        .with_actor(Iri::from(person.id().clone()))
        .with_object(Item::object(repository)))
}

pub(crate) async fn create_repository(
    app: &App,
    http_client: &reqwest::Client,
    oauth_token: &OAuthToken,
    person: &DbPerson,
    factory: &DbFactory,
    repository_name: Name,
) -> Result<(DbRepository, Create)> {
    let create = create_repository_request(app, person, repository_name).await?;

    let factory_id = factory.id();
    let create_uri = format!("{factory_id}/outbox");
    log::debug!("create repository URI: {create_uri}");

    let token = oauth_token.token();
    let body = create.to_string();

    let res = http_client
        .post(create_uri)
        .header(
            header::CONTENT_TYPE,
            MimeType::ApplicationActivityJson.as_str(),
        )
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(body)
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::OK);

    let repository = res.json::<Repository>().await?;
    let repository_id = repository.id().unwrap();

    DbRepository::find_by_id(&*app.state().db().await, &repository_id.into())
        .await
        .and_then(|f| {
            f.ok_or(Error::db(format!(
                "create_repository: no record for ID: {repository_id}"
            )))
        })
        .map(|r| (r, create))
}

pub(crate) async fn get_repository(
    http_client: &reqwest::Client,
    oauth_token: &OAuthToken,
    id: &DbIri,
) -> Result<Repository> {
    let token = oauth_token.token_header(OAuthTokenType::Access)?;

    let res = http_client
        .get(id.as_str())
        .header(
            header::CONTENT_TYPE,
            MimeType::ApplicationActivityJson.as_str(),
        )
        .header(header::AUTHORIZATION, token)
        .send()
        .await
        .map_err(|err| Error::http(format!("test: repository: {err}")))?;

    assert_eq!(res.status(), StatusCode::OK);

    res.json::<Repository>()
        .await
        .map_err(|err| Error::http(format!("tests: repository: {err}")))
}

pub(crate) async fn add_server_repository_like(
    key: &DbKey,
    inbox_id: &DbIri,
    actor: &DbIri,
    object: &DbIri,
) -> Result<Activity> {
    let privkey = HttpPrivateKey::try_from(key)?;

    let like = Activity::like(
        Like::new()
            .with_actor(Iri::from(actor))
            .with_object(Iri::from(object)),
    );

    log::info!("router: adding repository inbox activity to ID: {actor}");

    let res = App::signed_request_with_keys(&[privkey], Method::POST, inbox_id, Some(&like))
        .await
        .map_err(|err| {
            log::error!("error parsing Like response: {err}");
            err
        })?;

    assert_eq!(res.status(), StatusCode::OK);

    Ok(like)
}

pub(crate) async fn add_client_repository_like(
    http_client: &reqwest::Client,
    oauth_token: &OAuthToken,
    outbox_id: &DbIri,
    actor: &DbIri,
    object: &DbIri,
) -> Result<Activity> {
    let token = oauth_token.token_header(OAuthTokenType::Access)?;

    let like = Activity::like(
        Like::new()
            .with_actor(Iri::from(actor))
            .with_object(Iri::from(object)),
    );

    log::info!("router: adding repository outbox activity to ID: {actor}");

    let body = like.to_string();

    let res = http_client
        .post(outbox_id.as_str())
        .header(
            header::CONTENT_TYPE,
            MimeType::ApplicationActivityJson.as_str(),
        )
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::AUTHORIZATION, token)
        .body(body)
        .send()
        .await
        .map_err(|err| Error::http(format!("test: repository: {err}")))?;

    assert_eq!(res.status(), StatusCode::OK);

    Ok(like)
}
