use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use activityforge::app::App;
use activityforge::app::oauth::OAuthToken;
use activityforge::crypto::{HttpPrivateKey, KeyType, PemPublicKey, PublicKey};
use activityforge::db::{Iri as DbIri, Key as DbKey};
use activityforge::{Error, Result};
use activitystreams_vocabulary::{
    Collection, Iri, Key as VocabPemPublicKey, MultibaseHeader, MultibasePublicKey, Multikey,
    MultikeyPublicKey, Name, Person,
};

use axum::Router;
use axum::extract::{Path, Request};
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use http::{Method, StatusCode, header};

mod factory;
mod middleware;
mod oauth;
mod person;
mod repository;
mod signature;

const ED25519_PRIVKEY_BYTES: [u8; 32] = [
    0x73, 0x59, 0x10, 0x6e, 0x0a, 0x10, 0xcd, 0x1e, 0xb9, 0xda, 0xe1, 0xe5, 0xa0, 0xfc, 0x92, 0x39,
    0x19, 0x45, 0x75, 0x7a, 0xf8, 0x18, 0x54, 0xca, 0xec, 0x64, 0x6a, 0xf2, 0x73, 0xbf, 0x8b, 0x0b,
];

const ED25519_PUBKEY_BYTES: [u8; 32] = [
    0x7d, 0xb0, 0x56, 0xf4, 0xe, 0xa7, 0x10, 0xb6, 0x80, 0xa7, 0x6a, 0xd7, 0x26, 0xfd, 0xbd, 0x3e,
    0x70, 0xea, 0xd1, 0xd6, 0x20, 0xa7, 0x74, 0xdb, 0x4b, 0x1a, 0x2a, 0x48, 0xa0, 0xce, 0xa3, 0xe5,
];

// Represents the locally stored UUID for the "remote" signing user
const TEST_USER_UUID: &str = "5f64760c-8dbe-0c5e-0221-aa80559f2c80";

const TEST_USER_ID: &str =
    "http://127.0.0.1:4000/api/v1/persons/ca2bf7a3-e9d7-4c95-adf9-8ef8b774ff8e";

const ED25519_KEY_ID: &str =
    "http://127.0.0.1:4000/api/v1/keys/961d2c59-b4e3-4b8c-bbb8-7b30b6ec6486";

static MOCK_SERVER_STARTED: AtomicBool = AtomicBool::new(false);

static TEST_SERVER_PORT: AtomicU16 = AtomicU16::new(3000);

pub(crate) fn mock_server_started() -> bool {
    MOCK_SERVER_STARTED.fetch_or(true, Ordering::SeqCst)
}

pub(crate) async fn mock_server() -> Result<()> {
    if !mock_server_started() {
        let mock_server = Router::new()
            .route("/api/v1/keys/{key_id}", get(mock_key))
            .route("/api/v1/persons/{person_id}", get(mock_person));

        let mock_listener = tokio::net::TcpListener::bind("127.0.0.1:4000").await?;

        tokio::spawn(async move {
            axum::serve(mock_listener, mock_server).await.ok();
        });
    }

    Ok(())
}

pub(crate) fn test_server_port() -> u16 {
    TEST_SERVER_PORT.fetch_add(1, Ordering::SeqCst)
}

#[allow(deprecated)]
fn create_test_person() -> Person {
    let id = Iri::try_from(TEST_USER_ID).unwrap();
    let name = Name::try_from("test_user").unwrap();
    let inbox = Iri::try_from(format!("{id}/inbox")).unwrap();
    let outbox = Iri::try_from(format!("{id}/outbox")).unwrap();

    let multibase = MultibasePublicKey::new()
        .with_header(MultibaseHeader::Base64UrlNoPad)
        .with_key(MultikeyPublicKey::Ed25519(ED25519_PUBKEY_BYTES));

    let multikey = Multikey::new()
        .with_id(Iri::try_from(ED25519_KEY_ID).unwrap())
        .with_controller(Iri::try_from(TEST_USER_ID).unwrap())
        .with_public_key_multibase(multibase);

    let pubkey = PublicKey::from_bytes(KeyType::Ed25519, &ED25519_PUBKEY_BYTES).unwrap();

    let pemkey: VocabPemPublicKey = PemPublicKey::new()
        .with_id(Iri::try_from(ED25519_KEY_ID).unwrap())
        .with_owner(Iri::try_from(TEST_USER_ID).unwrap())
        .with_public_key_pem(pubkey)
        .try_into()
        .unwrap();

    Person::new()
        .with_id(id)
        .with_name(name)
        .with_inbox(inbox)
        .with_outbox(outbox)
        .with_assertion_method(multikey)
        .with_public_key(pemkey)
}

#[allow(deprecated)]
async fn mock_person(Path(person_id): Path<String>) -> Response {
    match person_id.as_str() {
        "ca2bf7a3-e9d7-4c95-adf9-8ef8b774ff8e" => Response::builder()
            .status(StatusCode::OK)
            .body(create_test_person().to_string().into())
            .unwrap(),
        _ => {
            log::error!("unknown person UUID: {person_id}");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

async fn mock_key(Path(key_id): Path<String>, request: Request) -> Response {
    let encoding = request
        .headers()
        .get("key-encoding")
        .map(|v| v.to_str().unwrap());

    match (key_id.as_str(), encoding) {
        ("961d2c59-b4e3-4b8c-bbb8-7b30b6ec6486", Some("multikey")) => {
            let multibase = MultibasePublicKey::new()
                .with_header(MultibaseHeader::Base64UrlNoPad)
                .with_key(MultikeyPublicKey::Ed25519(ED25519_PUBKEY_BYTES));

            let multikey = Multikey::new()
                .with_id(Iri::try_from(ED25519_KEY_ID).unwrap())
                .with_controller(Iri::try_from(TEST_USER_ID).unwrap())
                .with_public_key_multibase(multibase);

            log::debug!("test-user multikey Response: {multikey}");

            Response::builder()
                .status(StatusCode::OK)
                .body(multikey.to_string().into())
                .unwrap()
        }
        ("961d2c59-b4e3-4b8c-bbb8-7b30b6ec6486", Some("pem") | None) => {
            let Ok(pubkey) = PublicKey::from_bytes(KeyType::Ed25519, &ED25519_PUBKEY_BYTES)
                .map_err(|err| {
                    log::error!("router_test: error converting public key: {err}");
                })
            else {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            };

            let id = Iri::try_from(ED25519_KEY_ID).unwrap();
            let owner = Iri::try_from(TEST_USER_ID).unwrap();

            let pemkey = PemPublicKey::new()
                .with_id(id)
                .with_owner(owner)
                .with_public_key_pem(pubkey);

            log::debug!("test-user pemkey Response: {pemkey}");

            Response::builder()
                .status(StatusCode::OK)
                .body(pemkey.to_string().into())
                .unwrap()
        }
        (keyid, encoding) => {
            log::warn!("unknown key ID: {keyid}, encoding: {encoding:?}");
            StatusCode::NOT_IMPLEMENTED.into_response()
        }
    }
}

pub(crate) async fn get_mailbox(key: &DbKey, id: &DbIri) -> Result<Collection> {
    let privkey = HttpPrivateKey::try_from(key)?;

    let res = App::signed_request_with_keys::<()>(&[privkey], Method::GET, id, None)
        .await
        .map_err(|err| {
            log::error!("error parsing get_person response: {err}");
            err
        })?;

    assert_eq!(res.status(), StatusCode::OK);

    res.json::<Collection>().await.map_err(Error::from)
}

pub(crate) async fn get_client_mailbox(
    http_client: &reqwest::Client,
    oauth_token: &OAuthToken,
    id: &DbIri,
) -> Result<Collection> {
    let token = oauth_token.token();

    log::debug!("mailbox: getting mailbox at: {id}");

    let res = http_client
        .get(id.as_str())
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::OK);

    res.json::<Collection>().await.map_err(Error::from)
}
