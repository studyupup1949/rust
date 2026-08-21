#![allow(deprecated)]

use activityforge::app::App;
use activityforge::app::oauth::{OAuthToken, OAuthTokenType};
use activityforge::crypto::{KeyType, PrivateKey};
use activityforge::db::{
    Factory as DbFactory, Iri as DbIri, Key as DbKey, Name as DbName, Person as DbPerson, TableType,
};
use activityforge::{Activity, ActorType, Error, Factory, Repository, Result, util};
use activitystreams_vocabulary::{Collection, Create, Iri, Item, MimeType, Multikey, Name};

use http::{StatusCode, header};

use super::create_test_person;
use crate::router::oauth::{create_oauth_token, register_oauth_client};
use crate::router::repository::create_repository;

crate::router_test! {
    create_factory => run_create_factory_test(db, app) {
        let factory_name = Name::try_from("test_create_factory")?;
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

        get_factory(&app, &http_client, &oauth_token, &db_factory).await?;

        Ok(())
    }
}

crate::router_test! {
    create_invalid_factory => run_create_invalid_factory_test(db, app) {
        let factory_name = Name::try_from("test_create_invalid_factory")?;
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

        create_invalid_factory(
            app,
            &http_client,
            &oauth_token,
            &person,
            factory_name,
        ).await
    }
}

crate::router_test! {
    create_repository => run_create_repository_test(db, app) {
        let factory_name = Name::try_from("test_create_repository_factory")?;
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

        let repository_name = Name::try_from("test_create_repository")?;

        let (_db_repository, _create) = create_repository(
            &app,
            &http_client,
            &oauth_token,
            &person,
            &db_factory,
            repository_name,
        ).await?;

        Ok(())
    }
}

crate::router_test! {
    get_factory_outbox => run_get_factory_outbox_test(db, app) {
        let http_client = reqwest::Client::new();

        let factory_name = Name::try_from("test-get-factory-outbox")?;
        let owner_name = DbName::try_from(format!("{factory_name}-owner"))?;

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

        let factory_id = db_factory.id();
        let outbox_id = DbIri::try_from(format!("{factory_id}/outbox"))?;
        let activities = get_factory_outbox(&http_client, &oauth_token, &outbox_id).await?;

        let repository_name = Name::try_from("test_factory_get_outbox_repository")?;

        let (_db_repository, mut create) = create_repository(
            &app,
            &http_client,
            &oauth_token,
            &person,
            &db_factory,
            repository_name,
        ).await?;

        let activities = get_factory_outbox(&http_client, &oauth_token, &outbox_id).await?;

        assert_eq!(activities.total_items(), Some(1u64));

        let out_activity: Activity = activities.items().unwrap().try_into()?;
        let out_create = out_activity.into_create()?;
        let out_repo: Repository = out_create.object().unwrap().try_into()?;

        // update Create activity with values updated during DB storage
        create.set_id(out_create.id().unwrap().clone());

        assert_eq!(
            out_create.actor().unwrap().ids().unwrap(),
            create.actor().unwrap().ids().unwrap()
        );

        let repo: Repository = Repository::try_from(create.object().unwrap()).map(|f| {
            f.with_id(out_repo.id().unwrap().clone())
                .with_inbox(out_repo.inbox().unwrap().clone())
                .with_outbox(out_repo.outbox().unwrap().clone())
                .with_assertion_method(out_repo.assertion_method().unwrap().clone())
                .with_public_key(out_repo.public_key().unwrap().clone())
        })?;

        create.set_actor(out_create.actor().unwrap().as_single().unwrap().clone());
        create.set_object(repo);

        assert_eq!(out_create, create, "[{out_create}, {create}]");

        Ok(())
    }
}

pub(crate) async fn create_factory_request(
    app: &App,
    person: &DbPerson,
    factory_name: Name,
) -> Result<(Create, DbKey)> {
    let available_actor_types = [
        ActorType::Repository,
        ActorType::PatchTracker,
        ActorType::ReleaseTracker,
        ActorType::Roadmap,
        ActorType::TicketTracker,
        ActorType::Project,
        ActorType::Team,
        ActorType::Workflow,
    ];

    let factory_key_uuid = util::rand_uuid();
    let factory_key_id = TableType::Key.id_from_uuid(&app.uri(), factory_key_uuid)?;
    let factory_key = PrivateKey::random(KeyType::Ed25519)
        .and_then(DbKey::try_from)
        .map(|k| k.with_uuid(factory_key_uuid).with_id(factory_key_id))?;

    let factory_multikey = Multikey::try_from(&factory_key)?;

    let factory = Factory::new()
        .with_name(factory_name)
        .with_available_actor_types(available_actor_types)
        .with_assertion_method(factory_multikey);

    Ok((
        Create::new()
            .with_actor(Iri::from(person.id().clone()))
            .with_object(Item::object(factory)),
        factory_key,
    ))
}

pub(crate) async fn create_factory(
    app: &App,
    http_client: &reqwest::Client,
    oauth_token: &OAuthToken,
    person: &DbPerson,
    factory_name: Name,
) -> Result<(DbFactory, DbKey)> {
    let (create, key) = create_factory_request(app, person, factory_name).await?;

    let person_id = person.id();

    let create_uri = format!("{person_id}/outbox");
    log::debug!("create factory URI: {create_uri}");

    let body = create.to_string();
    let token = oauth_token.token_header(OAuthTokenType::Access)?;

    let res = http_client
        .post(create_uri)
        .header(
            header::CONTENT_TYPE,
            MimeType::ApplicationActivityJson.as_str(),
        )
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::AUTHORIZATION, token)
        .body(body)
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::OK);

    let factory = res
        .json::<Factory>()
        .await
        .map_err(|err| Error::http(format!("test: factory: {err}")))?;

    let factory_id = factory.id().ok_or(Error::http("factory: missing ID"))?;

    DbFactory::find_by_id(&*app.state().db().await, &factory_id.into())
        .await
        .and_then(|f| {
            f.ok_or(Error::db(format!(
                "create_factory: no record for ID: {factory_id}"
            )))
        })
        .map(|f| (f, key))
}

async fn create_invalid_factory(
    app: &App,
    http_client: &reqwest::Client,
    oauth_token: &OAuthToken,
    person: &DbPerson,
    factory_name: Name,
) -> Result<()> {
    // test that a mismatch of the Create actor and the signing actor results in an error
    let test_person =
        DbPerson::try_from_vocab(&*app.state().db().await, &create_test_person()).await?;
    let (create, _key) = create_factory_request(app, &test_person, factory_name).await?;

    let token = oauth_token.token_header(OAuthTokenType::Access)?;
    let outbox = format!("{}/outbox", person.id());
    let body = create.to_string();

    log::debug!("create factory URI: {outbox}");

    let res = http_client
        .post(outbox)
        .header(
            header::CONTENT_TYPE,
            MimeType::ApplicationActivityJson.as_str(),
        )
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::AUTHORIZATION, token)
        .body(body)
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

async fn get_factory(
    app: &App,
    http_client: &reqwest::Client,
    oauth_token: &OAuthToken,
    db_factory: &DbFactory,
) -> Result<()> {
    let token = oauth_token.token_header(OAuthTokenType::Access)?;

    let res = http_client
        .get(db_factory.id().as_str())
        .header(
            header::CONTENT_TYPE,
            MimeType::ApplicationActivityJson.as_str(),
        )
        .header(header::AUTHORIZATION, token)
        .send()
        .await
        .map_err(|err| Error::http(format!("test: factory: {err}")))?;

    assert_eq!(res.status(), StatusCode::OK);

    let res_factory = res.json::<Factory>().await?;
    let factory = db_factory.try_into_vocab(&*app.state().db().await).await?;

    assert_eq!(res_factory, factory);

    Ok(())
}

async fn get_factory_outbox(
    http_client: &reqwest::Client,
    oauth_token: &OAuthToken,
    id: &DbIri,
) -> Result<Collection> {
    log::info!("router: fetching factory from ID: {id}");

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
        .map_err(|err| Error::http(format!("test: factory: {err}")))?;

    assert_eq!(res.status(), StatusCode::OK);

    res.json::<Collection>()
        .await
        .map_err(|err| Error::http(format!("test: factory: {err}")))
}
