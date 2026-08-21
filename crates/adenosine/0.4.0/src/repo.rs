use crate::car::{
    load_car_bytes_to_blockstore, load_car_path_to_blockstore, read_car_bytes_from_blockstore,
};
use crate::crypto::KeyPair;
use crate::identifiers::{Did, Nsid, Tid};
use crate::mst::{collect_mst_keys, generate_mst, SignedCommitNode, UnsignedCommitNode};
use anyhow::{anyhow, ensure, Context, Result};
use ipfs_sqlite_block_store::BlockStore;
use libipld::cbor::DagCborCodec;
use libipld::multihash::Code;
use libipld::prelude::Codec;
use libipld::store::DefaultParams;
use libipld::{Block, Cid, Ipld};
use serde_json::{json, Value};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, serde::Serialize)]
pub struct RepoCommit {
    pub did: Did,
    pub version: u8,
    pub prev: Option<Cid>,
    pub mst_cid: Cid,
    pub sig: Box<[u8]>,
    pub commit_cid: Cid,
}

impl RepoCommit {
    /// Returns a JSON object version of this struct, with CIDs and signatures in expected format
    /// (aka, CID as a string, not an array of bytes).
    pub fn to_pretty_json(&self) -> Value {
        json!({
            "did": self.did.to_string(),
            "version": self.version.to_string(),
            "prev": self.prev.map(|v| v.to_string()),
            "mst_cid": self.mst_cid.to_string(),
            "sig": data_encoding::HEXUPPER.encode(&self.sig),
            "commit_cid": self.commit_cid.to_string(),
        })
    }
}

pub struct RepoStore {
    // TODO: only public for test/debug; should wrap instead
    pub db: BlockStore<libipld::DefaultParams>,
}

pub enum Mutation {
    Create(Nsid, Tid, Ipld),
    Update(Nsid, Tid, Ipld),
    Delete(Nsid, Tid),
}

impl RepoStore {
    pub fn open(db_path: &PathBuf) -> Result<Self> {
        Ok(RepoStore {
            db: BlockStore::open(db_path, Default::default())?,
        })
    }

    pub fn open_ephemeral() -> Result<Self> {
        Ok(RepoStore {
            db: BlockStore::open_path(ipfs_sqlite_block_store::DbPath::Memory, Default::default())?,
        })
    }

    pub fn new_connection(&mut self) -> Result<Self> {
        Ok(RepoStore {
            db: self.db.additional_connection()?,
        })
    }

    pub fn get_ipld(&mut self, cid: &Cid) -> Result<Ipld> {
        if let Some(b) = self.db.get_block(cid)? {
            let block: Block<DefaultParams> = Block::new(*cid, b)?;
            block.ipld()
        } else {
            Err(anyhow!("missing IPLD CID: {}", cid))
        }
    }

    pub fn get_blob(&mut self, cid: &Cid) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get_block(cid)?)
    }

    /// Returns CID that was inserted
    pub fn put_ipld<S: libipld::codec::Encode<DagCborCodec>>(&mut self, record: &S) -> Result<Cid> {
        let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, record)?;
        let cid = *block.cid();
        self.db
            .put_block(block, None)
            .context("writing IPLD DAG-CBOR record to blockstore")?;
        Ok(cid)
    }

    /// Returns CID that was inserted
    pub fn put_blob(&mut self, data: &[u8]) -> Result<Cid> {
        let block = Block::<DefaultParams>::encode(libipld::raw::RawCodec, Code::Sha2_256, data)?;
        let cid = *block.cid();
        self.db
            .put_block(block, None)
            .context("writing non-record blob to blockstore")?;
        Ok(cid)
    }

    /// Quick alias lookup
    pub fn lookup_commit(&mut self, did: &Did) -> Result<Option<Cid>> {
        Ok(self.db.resolve(Cow::from(did.as_bytes()))?)
    }

    pub fn get_commit(&mut self, commit_cid: &Cid) -> Result<RepoCommit> {
        // read records by CID: commit, root, meta
        let commit_node: SignedCommitNode = DagCborCodec
            .decode(
                &self
                    .db
                    .get_block(commit_cid)?
                    .ok_or(anyhow!("expected commit block in store"))?,
            )
            .context("parsing commit IPLD node from blockstore")?;
        ensure!(
            commit_node.version == 2,
            "unexpected repo version: {}",
            commit_node.version
        );
        Ok(RepoCommit {
            did: Did::from_str(&commit_node.did)?,
            version: commit_node.version,
            prev: commit_node.prev,
            mst_cid: commit_node.data,
            sig: commit_node.sig,
            commit_cid: *commit_cid,
        })
    }

    pub fn get_mst_record_by_key(&mut self, mst_cid: &Cid, key: &str) -> Result<Option<Ipld>> {
        let map = self.mst_to_map(mst_cid)?;
        if let Some(cid) = map.get(key) {
            self.get_ipld(cid).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn collections(&mut self, did: &Did) -> Result<Vec<String>> {
        let commit = if let Some(c) = self.lookup_commit(did)? {
            self.get_commit(&c)?
        } else {
            return Err(anyhow!("DID not found in repositories: {}", did));
        };
        let map = self.mst_to_map(&commit.mst_cid)?;
        let mut collections: HashSet<String> = Default::default();
        // XXX: confirm that keys actually start with leading slash
        for k in map.keys() {
            let coll = k.split('/').nth(1).unwrap();
            collections.insert(coll.to_string());
        }
        Ok(collections.into_iter().collect())
    }

    pub fn get_atp_record(
        &mut self,
        did: &Did,
        collection: &Nsid,
        tid: &Tid,
    ) -> Result<Option<Ipld>> {
        let commit = if let Some(c) = self.lookup_commit(did)? {
            self.get_commit(&c)?
        } else {
            return Ok(None);
        };
        let record_key = format!("{collection}/{tid}");
        self.get_mst_record_by_key(&commit.mst_cid, &record_key)
    }

    pub fn write_commit(
        &mut self,
        did: &Did,
        prev: Option<Cid>,
        mst_cid: Cid,
        signing_key: &KeyPair,
    ) -> Result<Cid> {
        let unsigned_commit = UnsignedCommitNode {
            did: did.to_string(),
            version: 2,
            prev,
            data: mst_cid,
        };
        let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &unsigned_commit)?;
        let digest = sha256::digest(block.data());
        let sig = signing_key.sign_bytes(digest.as_bytes());

        let commit_cid = self.put_ipld(&SignedCommitNode {
            did: did.to_string(),
            version: 2,
            prev,
            data: mst_cid,
            sig: sig.as_bytes().to_vec().into_boxed_slice(),
        })?;
        self.db.alias(did.as_bytes().to_vec(), Some(&commit_cid))?;
        Ok(commit_cid)
    }

    pub fn mst_from_map(&mut self, map: &BTreeMap<String, Cid>) -> Result<Cid> {
        let mst_cid = generate_mst(&mut self.db, map)?;
        Ok(mst_cid)
    }

    pub fn mst_to_map(&mut self, mst_cid: &Cid) -> Result<BTreeMap<String, Cid>> {
        let mut cid_map: BTreeMap<String, Cid> = Default::default();
        collect_mst_keys(&mut self.db, mst_cid, &mut cid_map)
            .context("reading repo MST from blockstore")?;
        Ok(cid_map)
    }

    pub fn update_mst(&mut self, mst_cid: &Cid, mutations: &[Mutation]) -> Result<Cid> {
        let mut cid_map = self.mst_to_map(mst_cid)?;
        for m in mutations.iter() {
            match m {
                Mutation::Create(collection, tid, val) => {
                    let cid = self.put_ipld(val)?;
                    cid_map.insert(format!("{collection}/{tid}"), cid);
                }
                Mutation::Update(collection, tid, val) => {
                    let cid = self.put_ipld(val)?;
                    cid_map.insert(format!("{collection}/{tid}"), cid);
                }
                Mutation::Delete(collection, tid) => {
                    cid_map.remove(&format!("{collection}/{tid}"));
                }
            }
        }
        let mst_cid = generate_mst(&mut self.db, &cid_map)?;
        Ok(mst_cid)
    }

    /// High-level helper to write a batch of mutations to the repo corresponding to the DID, and
    /// signing the resulting new root CID with the given keypair.
    pub fn mutate_repo(
        &mut self,
        did: &Did,
        mutations: &[Mutation],
        signing_key: &KeyPair,
    ) -> Result<Cid> {
        let commit_cid = self.lookup_commit(did)?.unwrap();
        let last_commit = self.get_commit(&commit_cid)?;
        let new_mst_cid = self
            .update_mst(&last_commit.mst_cid, mutations)
            .context("updating MST in repo")?;
        self.write_commit(did, Some(last_commit.commit_cid), new_mst_cid, signing_key)
    }

    /// Reads in a full MST tree starting at a repo commit, then re-builds and re-writes the tree
    /// in to the repo, and verifies that both the MST root CIDs are identical.
    pub fn verify_repo_mst(&mut self, commit_cid: &Cid) -> Result<()> {
        // load existing commit and MST tree
        let existing_commit = self.get_commit(commit_cid)?;
        let repo_map = self.mst_to_map(&existing_commit.mst_cid)?;

        // write MST tree, and verify root CID
        let new_mst_cid = self.mst_from_map(&repo_map)?;
        if new_mst_cid != existing_commit.mst_cid {
            Err(anyhow!(
                "MST root CID did not verify: {} != {}",
                existing_commit.mst_cid,
                new_mst_cid
            ))?;
        }

        Ok(())
    }

    /// Import blocks from a CAR file in memory, optionally setting an alias pointing to the input
    /// (eg, a DID identifier).
    ///
    /// Does not currently do any validation of, eg, signatures. It is naive and incomplete to use
    /// this to simply import CAR content from users, remote servers, etc.
    ///
    /// Returns the root commit from the CAR file, which may or may not actually be a "commit"
    /// block.
    pub fn import_car_bytes(&mut self, car_bytes: &[u8], alias: Option<String>) -> Result<Cid> {
        let cid = load_car_bytes_to_blockstore(&mut self.db, car_bytes)?;
        self.verify_repo_mst(&cid)?;
        if let Some(alias) = alias {
            self.db.alias(alias.as_bytes().to_vec(), Some(&cid))?;
        }
        Ok(cid)
    }

    /// Similar to import_car_bytes(), but reads from a local file on disk instead of from memory.
    pub fn import_car_path(&mut self, car_path: &PathBuf, alias: Option<String>) -> Result<Cid> {
        let cid = load_car_path_to_blockstore(&mut self.db, car_path)?;
        self.verify_repo_mst(&cid)?;
        if let Some(alias) = alias {
            self.db.alias(alias.as_bytes().to_vec(), Some(&cid))?;
        }
        Ok(cid)
    }

    /// Exports in CAR format to a Writer
    ///
    /// The "from" commit CID feature is not implemented.
    pub fn export_car(
        &mut self,
        commit_cid: &Cid,
        _from_commit_cid: Option<&Cid>,
    ) -> Result<Vec<u8>> {
        // TODO: from_commit_cid
        read_car_bytes_from_blockstore(&mut self.db, commit_cid)
    }
}

#[test]
fn test_repo_mst() {
    use libipld::ipld;

    let mut repo = RepoStore::open_ephemeral().unwrap();
    let did = Did::from_str("did:plc:dummy").unwrap();
    let dummy_keypair = KeyPair::new_random();

    // basic blob and IPLD record put/get
    let blob = b"beware the swamp thing";
    let blob_cid = repo.put_blob(blob).unwrap();

    let record = ipld!({"some-thing": 123});
    let record_cid = repo.put_ipld(&record).unwrap();

    repo.get_blob(&blob_cid).unwrap().unwrap();
    repo.get_ipld(&record_cid).unwrap();

    // basic MST get/put
    let mut map: BTreeMap<String, Cid> = Default::default();
    let empty_map_cid = repo.mst_from_map(&map).unwrap();
    assert_eq!(map, repo.mst_to_map(&empty_map_cid).unwrap());
    assert!(repo
        .get_mst_record_by_key(&empty_map_cid, "test.records/44444444444444")
        .unwrap()
        .is_none());

    map.insert("blobs/1".to_string(), blob_cid);
    map.insert("blobs/2".to_string(), blob_cid);
    map.insert("test.records/44444444444444".to_string(), record_cid);
    map.insert("test.records/22222222222222".to_string(), record_cid);
    let simple_map_cid = repo.mst_from_map(&map).unwrap();
    assert_eq!(map, repo.mst_to_map(&simple_map_cid).unwrap());

    // create root and commit IPLD nodes
    let simple_commit_cid = repo
        .write_commit(&did, None, simple_map_cid, &dummy_keypair)
        .unwrap();
    assert_eq!(
        Some(record.clone()),
        repo.get_mst_record_by_key(&simple_map_cid, "test.records/44444444444444")
            .unwrap()
    );
    assert_eq!(
        Some(record.clone()),
        repo.get_atp_record(
            &did,
            &Nsid::from_str("test.records").unwrap(),
            &Tid::from_str("44444444444444").unwrap()
        )
        .unwrap()
    );
    assert!(repo
        .get_mst_record_by_key(&simple_map_cid, "test.records/33333333333333")
        .unwrap()
        .is_none());
    assert!(repo
        .get_atp_record(
            &did,
            &Nsid::from_str("test.records").unwrap(),
            &Tid::from_str("33333333333333").unwrap()
        )
        .unwrap()
        .is_none());
    assert_eq!(Some(simple_commit_cid), repo.lookup_commit(&did).unwrap());

    map.insert("test.records/33333333333333".to_string(), record_cid);
    let simple3_map_cid = repo.mst_from_map(&map).unwrap();
    let simple3_commit_cid = repo
        .write_commit(
            &did,
            Some(simple_commit_cid),
            simple3_map_cid,
            &dummy_keypair,
        )
        .unwrap();
    assert_eq!(map, repo.mst_to_map(&simple3_map_cid).unwrap());
    assert_eq!(
        Some(record.clone()),
        repo.get_mst_record_by_key(&simple3_map_cid, "test.records/33333333333333")
            .unwrap()
    );
    assert_eq!(
        Some(record.clone()),
        repo.get_atp_record(
            &did,
            &Nsid::from_str("test.records").unwrap(),
            &Tid::from_str("33333333333333").unwrap()
        )
        .unwrap()
    );
    let commit = repo.get_commit(&simple3_commit_cid).unwrap();
    assert_eq!(commit.did, did);
    assert_eq!(commit.prev, Some(simple_commit_cid));
    assert_eq!(commit.mst_cid, simple3_map_cid);
    assert_eq!(
        Some(simple3_commit_cid.clone()),
        repo.lookup_commit(&did).unwrap()
    );
}
