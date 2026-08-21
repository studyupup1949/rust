#![allow(deprecated)]

use activityforge::app::App;
use activityforge::app::oauth::OAuthToken;
use activityforge::crypto::{AlgorithmName, HttpPrivateKey, Password};
use activityforge::db::{
    Actor as DbActor, Application as DbApplication, Db, Iri as DbIri, Key as DbKey, Name as DbName,
    Person as DbPerson, TableEntry, TableType, Uuid,
};
use activityforge::{Activity, Error, Factory, Result, Role, util};

use activitystreams_vocabulary::{
    Accept, Follow, Iri, Key as PemPublicKey, MimeType, Multikey, Name, Person,
};

use http::{Method, StatusCode, header};

use super::{
    ED25519_KEY_ID, ED25519_PRIVKEY_BYTES, TEST_USER_ID, TEST_USER_UUID, create_test_person,
    get_mailbox,
};

use crate::router::factory::create_factory;
use crate::router::get_client_mailbox;
use crate::router::oauth::{create_oauth_token, register_oauth_client};

crate::router_test! {
    get_person => run_get_person_test(db, app) {
        let test_user_id = DbIri::try_from(TEST_USER_ID)?;

        let person = create_test_person();
        if let Err(err) = DbPerson::try_from_vocab(db, &person).await {
            log::warn!("test: person: {err}");
        }

        get_person(&test_user_id, &person).await?;

        let remote_id = DbIri::try_from("https://example.dev/test-get-remote-person")?;
        let remote_name = DbName::try_from("test-get-remote-person")?;
        let (remote_person, _remote_db_person, _key) =
            create_remote_person(&app, &remote_id, &remote_name).await?;

        let local_remote_uuid = TableType::Person.uuid_from_id(&remote_id);
        let local_remote_id = TableType::Person.id_from_uuid(app.uri(), local_remote_uuid)?;

        log::info!("tests: router: looking up remote record: {local_remote_id}");

        get_person(&local_remote_id, &remote_person).await?;

        Ok(())
    }
}

crate::router_test! {
    get_server_mailbox => run_get_server_mailbox_test(db, app) {
        let http_client = reqwest::Client::new();

        let name = DbName::try_from("test-get-server-mailbox")?;
        let password = "super-secret-password";

        let (db_person, _client_key, client_res) = register_oauth_client(
            app,
            db,
            &http_client,
            &name,
            password,
        ).await?;

        let person = db_person.try_into_vocab(db).await?;

        let key = db_person
            .keys(db)
            .await
            .and_then(|k| k.first().cloned().ok_or(Error::db("test: person: missing keys")))?;

        let client_id = client_res.client_id();
        let client_secret = client_res.client_secret();

        let oauth_token = create_oauth_token(
            app,
            &http_client,
            &client_id,
            client_secret,
            b"super-secret-pkce",
        ).await?;

        get_person(db_person.id(), &person).await?;

        let id = db_person.id();
        let outbox_id = DbIri::try_from(format!("{id}/outbox"))?;
        let activities = get_mailbox(&key, &outbox_id).await?;

        assert_eq!(activities.total_items(), Some(0u64));

        let factory_name = Name::try_from("test_get_person_mailbox_create_factory")?;
        let (db_factory, _key) = create_factory(
            app,
            &http_client,
            &oauth_token,
            &db_person,
            factory_name,
        ).await?;

        let factory = db_factory.try_into_vocab(db).await?;

        let activities = get_mailbox(&key, &outbox_id).await?;

        assert_eq!(activities.total_items(), Some(1u64));

        let out_activity: Activity = activities.items().unwrap().try_into()?;
        let out_create = out_activity.into_create()?;
        let out_factory: Factory = out_create.object().unwrap().try_into()?;

        assert_eq!(out_factory.id().unwrap(), factory.id().unwrap(), "[{out_factory}, {factory}]");

        Ok(())
    }
}

crate::router_test! {
    get_client_mailbox => run_get_client_mailbox_test(db, app) {
        let http_client = reqwest::Client::new();

        let name = DbName::try_from("test-get-client-mailbox")?;
        let password = "super-secret-password";

        let (person, _client_privkey, client_res) = register_oauth_client(
            app,
            db,
            &http_client,
            &name,
            password,
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

        let person_id = person.id();

        log::debug!("test: person: fetching person {person_id} with OAuth client: {client_id}");

        let _vocab_person = get_client_person(&http_client, person_id, &oauth_token).await?;

        let inbox_id = DbIri::try_from(format!("{person_id}/inbox"))?;
        let outbox_id = DbIri::try_from(format!("{person_id}/outbox"))?;

        let in_activities = get_client_mailbox(&http_client, &oauth_token, &inbox_id).await?;
        assert_eq!(in_activities.total_items(), Some(0u64));

        let out_activities = get_client_mailbox(&http_client, &oauth_token, &outbox_id).await?;
        assert_eq!(out_activities.total_items(), Some(0u64));

        Ok(())
    }
}

crate::router_test! {
    person_follow => run_person_follow_test(db, app) {
        let follower_id = DbIri::try_from("https://example.dev/test-follower-person")?;
        let follower_name = DbName::try_from("test-follower-person")?;
        let (follower_person, db_follower_person, follower_key) =
            create_remote_person(&app, &follower_id, &follower_name).await?;

        let followed_id = DbIri::try_from("https://example.dev/test-followed-person")?;
        let followed_name = DbName::try_from("test-followed-person")?;
        let (followed_person, db_followed_person, followed_key) =
            create_remote_person(&app, &followed_id, &followed_name).await?;

        let follower_actor = DbActor::person(db_follower_person);

        app.state()
            .create_grant(
                &follower_actor,
                &[Role::Visit, Role::Write],
                db_followed_person.table_entry(),
            )
            .await?;

        app.state()
            .create_grant(
                &follower_actor,
                &[Role::Visit, Role::Write],
                TableEntry::create(TableType::Inbox, db_followed_person.inbox()),
            )
            .await?;

        let local_followed_uuid = TableType::Person.uuid_from_id(&followed_id);
        let local_followed_id = TableType::Person.id_from_uuid(app.uri(), local_followed_uuid)?;

        let inbox_id = DbIri::try_from(format!("{local_followed_id}/inbox"))?;
        let activities = get_mailbox(&followed_key, &inbox_id).await?;

        assert_eq!(activities.total_items(), Some(0u64));

        let _activity =
            add_person_follow(&follower_key, &inbox_id, &follower_person, &followed_person).await?;

        let activities = get_mailbox(&followed_key, &inbox_id).await?;

        assert_eq!(activities.total_items(), Some(1u64));

        let follow = Follow::try_from(activities.items().unwrap())?;

        assert_eq!(
            follow.actor().unwrap().ids().unwrap().first().unwrap(),
            &follower_person.id().unwrap()
        );
        assert_eq!(
            follow.object().unwrap().ids().unwrap().first().unwrap(),
            &followed_person.id().unwrap()
        );

        Ok(())
    }
}

pub(crate) async fn create_remote_person(
    app: &App,
    id: &DbIri,
    name: &DbName,
) -> Result<(Person, DbPerson, DbKey)> {
    let uuid = TableType::Person.uuid_from_id(id);

    let inbox = Iri::try_from(format!("{id}/inbox")).unwrap();
    let outbox = Iri::try_from(format!("{id}/outbox")).unwrap();

    let person_actor = TableEntry::create(TableType::Person, uuid);

    let key_uuid = util::rand_uuid();
    let key_id = TableType::Key.id_from_uuid(id, key_uuid)?;
    let key = HttpPrivateKey::random(key_id.as_str(), AlgorithmName::Ed25519)
        .and_then(DbKey::try_from)
        .map(|k| k.with_actor_id(id).with_actor(person_actor).with_id(key_id))?;

    let multikey: Multikey = (&key).try_into()?;
    let pemkey: PemPublicKey = (&key).try_into()?;

    let person = Person::new()
        .with_id(id)
        .with_name(name)
        .with_inbox(inbox)
        .with_outbox(outbox)
        .with_assertion_method(multikey)
        .with_public_key(pemkey);

    log::info!("tests: router: storing remote record: {person}");

    let db_person =
        DbPerson::try_from_vocab_with_uuid(&*app.state().db().await, &person, uuid).await?;

    log::info!("tests: router: successfully stored actor: {person}");

    let person_actor = DbActor::person(db_person.clone());
    let test_actor = DbActor::person(DbPerson::new().with_uuid(Uuid::parse_str(TEST_USER_UUID)?));
    let app_actor = DbActor::application(DbApplication::new().with_uuid(app.state().app().uuid()));

    app.state()
        .create_grant(
            &person_actor,
            &[Role::Visit, Role::Write],
            db_person.table_entry(),
        )
        .await?;

    app.state()
        .create_grant(
            &test_actor,
            &[Role::Visit, Role::Write],
            db_person.table_entry(),
        )
        .await?;

    app.state()
        .create_grant(&app_actor, &[Role::Visit], db_person.table_entry())
        .await
        .map(|_| (person, db_person, key))
}

pub(crate) async fn get_person(id: &DbIri, person: &Person) -> Result<()> {
    let privkey = HttpPrivateKey::from_bytes(
        ED25519_KEY_ID,
        AlgorithmName::Ed25519,
        &ED25519_PRIVKEY_BYTES,
    )?;

    let keys = [privkey];

    log::info!("router: fetching person from ID: {id}");

    let res = App::signed_request_with_keys::<()>(&keys, Method::GET, id, None)
        .await
        .map_err(|err| {
            log::error!("error parsing get_person response: {err}");
            err
        })?;

    assert_eq!(res.status(), StatusCode::OK);

    let body = res.text().await?;

    assert_eq!(
        serde_json::from_str::<Person>(&body).as_ref().unwrap(),
        person,
    );

    Ok(())
}

pub(crate) async fn add_person_follow(
    key: &DbKey,
    mailbox_id: &DbIri,
    follower_person: &Person,
    followed_person: &Person,
) -> Result<Activity> {
    let privkey = HttpPrivateKey::try_from(key)?;

    let follow = Activity::follow(
        Follow::new()
            .with_actor(follower_person.clone())
            .with_object(followed_person.id().cloned().unwrap()),
    );

    log::info!("tests: router: adding person mailbox activity to ID: {mailbox_id}");

    let res = App::signed_request_with_keys(&[privkey], Method::POST, mailbox_id, Some(&follow))
        .await
        .map_err(|err| {
            log::error!("tests: router: error parsing follow_person response: {err}");
            err
        })?;

    assert_eq!(res.status(), StatusCode::OK);

    res.json::<Accept>()
        .await
        .map(Activity::accept)
        .map_err(|err| {
            Error::http(format!(
                "tests: router: error parsing create_factory response: {err}"
            ))
        })
}

pub(crate) async fn create_local_person(
    db: &Db,
    uri: &DbIri,
    name: &DbName,
    password: &str,
) -> Result<DbPerson> {
    let person_uuid = util::rand_uuid();
    let person_id = DbPerson::TABLE.id_from_uuid(uri, person_uuid)?;
    let person_password = Password::derive(name.as_str(), password.as_bytes())?;

    let key_uuid = util::rand_uuid();
    let key_id = TableType::Key.id_from_uuid(uri, key_uuid)?;
    let key = HttpPrivateKey::random(key_id.as_str(), AlgorithmName::Ed25519)
        .and_then(DbKey::try_from)?;

    DbPerson::builder(person_id, name.clone())
        .map(|b| b.password(person_password))
        .and_then(|b| b.keys([key]))?
        .build(db)
        .await
}

pub(crate) async fn get_client_person(
    http_client: &reqwest::Client,
    id: &DbIri,
    oauth_token: &OAuthToken,
) -> Result<Person> {
    let token = oauth_token.token();

    let res = http_client
        .get(id.as_str())
        .header(
            header::CONTENT_TYPE,
            MimeType::ApplicationActivityJson.as_str(),
        )
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await
        .map_err(|err| Error::http(format!("test: person: {err}")))?;

    assert_eq!(res.status(), StatusCode::OK);

    res.json::<Person>()
        .await
        .map_err(|err| Error::http(format!("test: person: {err}")))
}
