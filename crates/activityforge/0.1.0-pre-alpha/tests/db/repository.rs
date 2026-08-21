use activityforge::Result;
use activityforge::db::activity::Like;
use activityforge::db::actor::Repository;
use activityforge::db::object::{Follower, Inbox, Key, Outbox};
use activityforge::db::{Db, Iri, KeyType, Name, TableEntry, TableType};

/// Tests database operations on [Repository] records.
pub async fn test_repository(db: &Db) -> Result<()> {
    let mut repo = create_repo(db, Name::try_from("test_repo")?).await?;

    test_repository_uris(db, &mut repo).await?;
    test_repository_forks(db, &mut repo).await?;
    test_repository_likes(db, &mut repo).await?;
    test_repository_followers(db, &mut repo).await?;
    test_repository_keys(db, &mut repo).await?;

    Ok(())
}

async fn create_repo(db: &Db, repo_name: Name) -> Result<Repository> {
    let repo_uuid = db.rand_uuid();
    let repo_id = Iri::try_from(format!("https://example.dev/respositories/{repo_name}"))?;

    let repo_inbox_id = Iri::try_from(format!("{repo_id}/inbox"))?;
    let mut repo_inbox = Inbox::new()
        .with_id(repo_inbox_id)
        .with_actor(TableEntry::create(TableType::Repository, repo_uuid));

    repo_inbox.insert(&db).await?;

    let repo_outbox_id = Iri::try_from(format!("{repo_id}/outbox"))?;
    let mut repo_outbox = Outbox::new()
        .with_id(repo_outbox_id)
        .with_actor(TableEntry::create(TableType::Repository, repo_uuid));

    repo_outbox.insert(&db).await?;

    let repo_key_id = Iri::try_from(format!("{repo_id}/keys#main-key"))?;

    let mut repo_key_data = [0u8; 64];
    rand::fill(&mut repo_key_data);

    let mut repo_key = Key::new()
        .with_id(repo_key_id)
        .with_key_type(KeyType::Ed25519)
        .with_key(repo_key_data)
        .with_is_private(true)
        .with_actor(TableEntry::create(TableType::Person, repo_uuid));

    repo_key.insert(db).await?;

    let mut repo = Repository::new()
        .with_uuid(repo_uuid)
        .with_id(repo_id)
        .with_name(repo_name)
        .with_inbox(repo_inbox.uuid())
        .with_outbox(repo_outbox.uuid())
        .with_key_ids([repo_key.uuid()])?;

    repo.insert(db).await?;

    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&repo));

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

    let repo_uuid = repo.uuid();
    let fork_uuid = fork_repo.uuid();

    repo.add_fork(db, fork_uuid).await?;
    assert_eq!(repo.forks().as_ref(), [fork_uuid].as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_fork(db, fork_uuid).await.is_err());
    assert!(repo.add_forks(db, [fork_uuid]).await.is_err());

    let next_fork_repo = create_repo(db, Name::try_from("test_second_fork_repo")?).await?;
    let next_fork_uuid = next_fork_repo.uuid();

    repo.add_fork(db, next_fork_uuid).await?;
    assert_eq!(repo.forks().as_ref(), [fork_uuid, next_fork_uuid].as_ref());
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
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()))
        .with_object(TableEntry::create(TableType::Repository, repo_uuid));

    like0.insert(db).await?;

    let like1_uuid = db.rand_uuid();

    let mut like1 = Like::new()
        .with_id(Iri::try_from(format!(
            "https://example.dev/likes/{like1_uuid}"
        ))?)
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()))
        .with_object(TableEntry::create(TableType::Repository, repo_uuid));

    like1.insert(db).await?;

    let mut likes = vec![like0_uuid, like1_uuid];
    likes.sort();

    repo.add_like(db, like0_uuid).await?;
    assert_eq!(repo.likes(), [like0_uuid].as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_like(db, like0_uuid).await.is_err());

    repo.delete_like(db, like0_uuid).await?;
    assert!(repo.likes().is_empty());

    repo.add_likes(db, [like0_uuid, like1_uuid]).await?;
    assert_eq!(repo.likes(), likes.as_slice());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_likes(db, [like0_uuid]).await.is_err());
    assert!(repo.add_likes(db, [like1_uuid]).await.is_err());
    assert!(repo.add_likes(db, [like0_uuid, like1_uuid]).await.is_err());

    repo.delete_likes(db, [like0_uuid, like1_uuid]).await?;
    assert!(repo.likes().is_empty());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    Ok(())
}

async fn test_repository_followers(db: &Db, repo: &mut Repository) -> Result<()> {
    let repo_uuid = repo.uuid();

    let mut follower0 = Follower::new()
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()))
        .with_following([TableEntry::create(TableType::Repository, repo_uuid)])?;

    follower0.insert(db).await?;

    let mut follower1 = Follower::new()
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()))
        .with_following([TableEntry::create(TableType::Repository, repo.uuid())])?;

    follower1.insert(db).await?;

    let follower0_uuid = follower0.uuid();
    let follower1_uuid = follower1.uuid();

    let mut followers = vec![follower0_uuid, follower1_uuid];
    followers.sort();

    repo.add_follower(db, follower0_uuid).await?;
    assert_eq!(repo.followers(), [follower0_uuid].as_ref());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_follower(db, follower0_uuid).await.is_err());

    repo.delete_follower(db, follower0_uuid).await?;
    assert!(repo.followers().is_empty());

    repo.add_followers(db, [follower0_uuid, follower1_uuid])
        .await?;
    assert_eq!(repo.followers(), followers.as_slice());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    // check duplicates are rejected
    assert!(repo.add_followers(db, [follower0_uuid]).await.is_err());
    assert!(repo.add_followers(db, [follower1_uuid]).await.is_err());
    assert!(
        repo.add_followers(db, [follower0_uuid, follower1_uuid])
            .await
            .is_err()
    );

    repo.delete_followers(db, [follower0_uuid, follower1_uuid])
        .await?;
    assert!(repo.followers().is_empty());
    assert_eq!(Repository::get(db, &repo_uuid).await.as_ref(), Ok(&*repo));

    Ok(())
}

async fn test_repository_keys(db: &Db, repo: &mut Repository) -> Result<()> {
    let repo_uuid = repo.uuid();
    let repo_id = repo.id().clone();

    let key_id = Iri::try_from(format!("{repo_id}/keys#client-key"))?;

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
