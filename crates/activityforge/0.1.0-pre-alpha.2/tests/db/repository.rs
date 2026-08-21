use activityforge::Result;
use activityforge::crypto::KeyType;
use activityforge::db::activity::Like;
use activityforge::db::actor::Repository;
use activityforge::db::object::{Follower, Key};
use activityforge::db::{Db, Iri, Name, TableEntry, TableType};

crate::db_test!(repository);

/// Tests database operations on [Repository] records.
async fn run_tests(db: &Db) -> Result<()> {
    let mut repo = create_repo(db, Name::try_from("test_repo")?).await?;

    test_repository_uris(db, &mut repo).await?;
    test_repository_forks(db, &mut repo).await?;
    test_repository_likes(db, &mut repo).await?;
    test_repository_followers(db, &mut repo).await?;
    test_repository_keys(db, &mut repo).await?;

    Ok(())
}

async fn create_repo(db: &Db, repo_name: Name) -> Result<Repository> {
    let host = "https://example.dev/api/v1";

    let repo_uuid = db.rand_uuid();
    let repo_id = Iri::try_from(format!("{host}/respositories/{repo_uuid}"))?;

    let repo_key_uuid = db.rand_uuid();
    let repo_key_id = Iri::try_from(format!("{host}/keys/{repo_key_uuid}"))?;

    let mut repo_key_data = [0u8; 64];
    rand::fill(&mut repo_key_data);

    let repo_key = Key::new()
        .with_uuid(repo_key_uuid)
        .with_id(repo_key_id.clone())
        .with_key_type(KeyType::Ed25519)
        .with_key(repo_key_data)
        .with_is_private(true);

    let repo = Repository::builder(repo_id, repo_name)
        .and_then(|b| b.uuid(repo_uuid))
        .and_then(|b| b.keys([repo_key]))?
        .build(db)
        .await?;

    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&repo));
    assert_eq!(
        Repository::find_by_key_id(db, &repo_key_id).await?.as_ref(),
        Some(&repo)
    );

    Ok(repo)
}

async fn test_repository_uris(db: &Db, repo: &mut Repository) -> Result<()> {
    let clone_uris = [
        Iri::try_from("https://example.dev/test_user/test_repo.git")?,
        Iri::try_from("ssh://git@example.dev:test_user/test_repo.git")?,
    ];

    let repo_uuid = repo.uuid();

    repo.add_clone_uris(db, clone_uris.clone()).await?;

    assert_eq!(repo.clone_uris(), clone_uris.as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_clone_uri(db, clone_uris[0].clone()).await.is_err());
    assert!(repo.add_clone_uri(db, clone_uris[1].clone()).await.is_err());
    assert!(repo.add_clone_uris(db, clone_uris.clone()).await.is_err());

    repo.delete_clone_uri(db, clone_uris[0].clone()).await?;
    assert_eq!(repo.clone_uris(), clone_uris[1..].as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    repo.delete_clone_uris(db, clone_uris.clone()).await?;
    assert!(repo.clone_uris().is_empty());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    let push_uris = [
        Iri::try_from("https://example.dev/test_user/test_repo.git")?,
        Iri::try_from("ssh://git@example.dev:test_user/test_repo.git")?,
    ];

    repo.add_push_uris(db, clone_uris.clone()).await?;

    assert_eq!(repo.push_uris(), push_uris.as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    repo.delete_push_uri(db, push_uris[0].clone()).await?;
    assert_eq!(repo.push_uris(), push_uris[1..].as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    repo.delete_push_uris(db, push_uris.clone()).await?;
    assert!(repo.push_uris().is_empty());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    Ok(())
}

async fn test_repository_forks(db: &Db, repo: &mut Repository) -> Result<()> {
    let fork_repo = create_repo(db, Name::try_from("test_fork_repo")?).await?;
    let fork_id = fork_repo.id();

    let repo_uuid = repo.uuid();

    repo.add_fork(db, fork_id.clone()).await?;
    assert_eq!(repo.forks().as_ref(), [fork_id.clone()].as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_fork(db, fork_id.clone()).await.is_err());
    assert!(repo.add_forks(db, [fork_id.clone()]).await.is_err());

    let next_fork_repo = create_repo(db, Name::try_from("test_second_fork_repo")?).await?;
    let next_fork_id = next_fork_repo.id();

    repo.add_fork(db, next_fork_id.clone()).await?;
    assert_eq!(
        repo.forks().as_ref(),
        [fork_id.clone(), next_fork_id.clone()].as_ref()
    );
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    Ok(())
}

async fn test_repository_likes(db: &Db, repo: &mut Repository) -> Result<()> {
    let repo_uuid = repo.uuid();

    let like0_uuid = db.rand_uuid();

    let mut like0 = Like::new()
        .with_id(Iri::try_from(format!(
            "https://example.dev/likes/{like0_uuid}"
        ))?)
        .with_actor(TableType::Person.id_from_uuid(repo.id(), db.rand_uuid())?)
        .with_object(repo.id());

    like0.insert(db).await?;

    let like1_uuid = db.rand_uuid();

    let mut like1 = Like::new()
        .with_id(Iri::try_from(format!(
            "https://example.dev/likes/{like1_uuid}"
        ))?)
        .with_actor(TableType::Person.id_from_uuid(repo.id(), db.rand_uuid())?)
        .with_object(repo.id().clone());

    like1.insert(db).await?;

    let mut likes = vec![like0.uuid(), like1.uuid()];
    likes.sort();

    repo.add_like(db, like0.uuid()).await?;
    assert_eq!(repo.likes(), [like0.uuid()].as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_like(db, like0.uuid()).await.is_err());

    repo.delete_like(db, like0.uuid()).await?;
    assert!(repo.likes().is_empty());

    repo.add_likes(db, likes.iter().copied()).await?;
    assert_eq!(repo.likes(), likes.as_slice());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_likes(db, [like0.uuid().clone()]).await.is_err());
    assert!(repo.add_likes(db, [like1.uuid().clone()]).await.is_err());
    assert!(repo.add_likes(db, likes.iter().copied()).await.is_err());

    repo.delete_likes(db, likes.iter().copied()).await?;
    assert!(repo.likes().is_empty());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    Ok(())
}

async fn test_repository_followers(db: &Db, repo: &mut Repository) -> Result<()> {
    let repo_uuid = repo.uuid();

    let follower0_uuid = db.rand_uuid();
    let follower0_id = TableType::Follower.id_from_uuid(repo.id(), follower0_uuid)?;

    let mut follower0 = Follower::new()
        .with_uuid(follower0_uuid)
        .with_id(follower0_id.clone())
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()))
        .with_following([TableEntry::create(TableType::Repository, repo_uuid)])?;

    follower0.insert(db).await?;

    let follower1_uuid = db.rand_uuid();
    let follower1_id = TableType::Follower.id_from_uuid(repo.id(), follower1_uuid)?;

    let mut follower1 = Follower::new()
        .with_uuid(follower1_uuid)
        .with_id(follower1_id.clone())
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()))
        .with_following([TableEntry::create(TableType::Repository, repo.uuid())])?;

    follower1.insert(db).await?;

    let mut followers = vec![follower0_id.clone(), follower1_id.clone()];
    followers.sort();

    repo.add_follower(db, follower0_id.clone()).await?;
    assert_eq!(repo.followers(), [follower0_id.clone()].as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_follower(db, follower0_id.clone()).await.is_err());

    repo.delete_follower(db, follower0_id.clone()).await?;
    assert!(repo.followers().is_empty());

    repo.add_followers(db, followers.clone()).await?;
    assert_eq!(repo.followers(), followers.as_slice());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(
        repo.add_followers(db, [follower0_id.clone()])
            .await
            .is_err()
    );
    assert!(
        repo.add_followers(db, [follower1_id.clone()])
            .await
            .is_err()
    );
    assert!(repo.add_followers(db, followers.clone()).await.is_err());

    repo.delete_followers(db, followers).await?;
    assert!(repo.followers().is_empty());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    Ok(())
}

async fn test_repository_keys(db: &Db, repo: &mut Repository) -> Result<()> {
    let repo_uuid = repo.uuid();

    let key_uuid = db.rand_uuid();
    let key_id = TableType::Key.id_from_uuid(repo.id(), key_uuid)?;

    let mut key_data = [0u8; 32];
    rand::fill(&mut key_data);

    let mut key = Key::new()
        .with_id(key_id)
        .with_key_type(KeyType::Ed25519)
        .with_key(key_data)
        .with_is_private(false)
        .with_actor(TableEntry::create(TableType::Repository, repo_uuid));

    key.insert(db).await?;

    let key_uuid = key.uuid();

    repo.add_key_id(db, key_uuid).await?;
    assert!(repo.key_ids().contains(&key_uuid));
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    let mut key_ids = repo.key_ids().to_vec();
    key_ids.sort();

    let key_id0_uuid = key_ids[0];
    let key_id1_uuid = key_ids[1];

    // check duplicates are rejected
    assert!(repo.add_key_id(db, key_id0_uuid).await.is_err());

    repo.delete_key_id(db, key_id0_uuid).await?;
    repo.delete_key_id(db, key_id1_uuid).await?;
    assert!(repo.key_ids().is_empty());

    repo.add_key_ids(db, [key_id0_uuid, key_id1_uuid]).await?;
    assert_eq!(repo.key_ids(), key_ids.as_slice());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_key_ids(db, [key_id0_uuid]).await.is_err());
    assert!(repo.add_key_ids(db, [key_id1_uuid]).await.is_err());
    assert!(
        repo.add_key_ids(db, [key_id0_uuid, key_id1_uuid])
            .await
            .is_err()
    );

    repo.delete_key_ids(db, [key_id0_uuid, key_id1_uuid])
        .await?;
    assert!(repo.key_ids().is_empty());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    Ok(())
}
