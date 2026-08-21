/// This is a simple immutable implemenation of of a Merkle Search Tree (MST) for atproto.
///
/// Mutations on the data structure are not implemented; instead the entire tree is read into a
/// BTreeMap, that is mutated, and then the entire tree is regenerated. This makes implementation
/// much simpler, at the obvious cost of performance in some situations.
///
/// The MST is basically a sorted key/value store where the key is a string and the value is a CID.
/// Tree nodes are stored as DAG-CBOG IPLD blocks, with references as CIDs.
///
/// In the atproto MST implementation, SHA-256 is the hashing algorithm, and "leading zeros" are
/// counted in blocks of 4 bits (so a leading zero byte counts as two zeros). This happens to match
/// simple hex encoding of the SHA-256 hash.
use anyhow::{anyhow, Context, Result};
use ipfs_sqlite_block_store::BlockStore;
use libipld::cbor::DagCborCodec;
use libipld::multihash::Code;
use libipld::prelude::Codec;
use libipld::store::DefaultParams;
use libipld::Block;
use libipld::{Cid, DagCbor};
use log::{debug, error, info};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, DagCbor, PartialEq, Eq)]
pub struct SignedCommitNode {
    pub did: String,
    pub version: u8,
    pub prev: Option<Cid>,
    pub data: Cid,
    pub sig: Box<[u8]>,
}

#[derive(Debug, DagCbor, PartialEq, Eq)]
pub struct UnsignedCommitNode {
    pub did: String,
    pub version: u8,
    pub prev: Option<Cid>,
    pub data: Cid,
}

#[derive(Debug, DagCbor, PartialEq, Eq)]
struct MstEntry {
    p: u32,
    k: Box<[u8]>,
    v: Cid,
    t: Option<Cid>,
}

#[derive(Debug, DagCbor, PartialEq)]
struct MstNode {
    l: Option<Cid>,
    e: Vec<MstEntry>,
}

struct WipEntry {
    height: u8,
    key: Box<[u8]>,
    val: Cid,
    right: Option<Box<WipNode>>,
}

struct WipNode {
    height: u8,
    left: Option<Box<WipNode>>,
    entries: Vec<WipEntry>,
}

fn get_mst_node(db: &mut BlockStore<libipld::DefaultParams>, cid: &Cid) -> Result<MstNode> {
    let block = &db
        .get_block(cid)?
        .ok_or(anyhow!("reading MST node from blockstore"))?;
    //println!("{:?}", block);
    let mst_node: MstNode = DagCborCodec
        .decode(block)
        .context("parsing MST DAG-CBOR IPLD node from blockstore")?;
    Ok(mst_node)
}

pub fn print_mst_keys(db: &mut BlockStore<libipld::DefaultParams>, cid: &Cid) -> Result<()> {
    let node = get_mst_node(db, cid)?;
    if let Some(ref left) = node.l {
        print_mst_keys(db, left)?;
    }
    let mut key: Vec<u8> = vec![];
    for entry in node.e.iter() {
        key = [&key[0..entry.p as usize], &entry.v.to_bytes()].concat();
        println!("\t{}\t-> {}", String::from_utf8_lossy(&key), entry.v);
        if let Some(ref right) = entry.t {
            print_mst_keys(db, right)?;
        }
    }
    Ok(())
}

pub fn dump_mst_keys(db_path: &PathBuf) -> Result<()> {
    let mut db: BlockStore<libipld::DefaultParams> = BlockStore::open(db_path, Default::default())?;

    let all_aliases: Vec<(Vec<u8>, Cid)> = db.aliases()?;
    if all_aliases.is_empty() {
        error!("expected at least one alias in block store");
        std::process::exit(-1);
    }

    // print all the aliases
    for (alias, commit_cid) in all_aliases.iter() {
        let did = String::from_utf8_lossy(alias);
        println!("{did} -> {commit_cid}");
    }

    let (did, commit_cid) = all_aliases[0].clone();
    let did = String::from_utf8_lossy(&did);
    info!("starting from {} [{}]", commit_cid, did);

    // NOTE: the faster way to develop would have been to decode to libipld::ipld::Ipld first? meh

    debug!(
        "raw commit: {:?}",
        &db.get_block(&commit_cid)?
            .ok_or(anyhow!("expected commit block in store"))?
    );
    let commit: SignedCommitNode = DagCborCodec.decode(
        &db.get_block(&commit_cid)?
            .ok_or(anyhow!("expected commit block in store"))?,
    )?;
    debug!("Commit: {:?}", commit);
    let mst_node: MstNode = DagCborCodec.decode(
        &db.get_block(&commit.data)?
            .ok_or(anyhow!("expected block in store"))?,
    )?;
    debug!("MST root node: {:?}", mst_node);
    debug!("============");

    println!("{did}");
    print_mst_keys(&mut db, &commit.data)?;
    Ok(())
}

pub fn collect_mst_keys(
    db: &mut BlockStore<libipld::DefaultParams>,
    cid: &Cid,
    map: &mut BTreeMap<String, Cid>,
) -> Result<()> {
    let node = get_mst_node(db, cid)?;
    if let Some(ref left) = node.l {
        collect_mst_keys(db, left, map)?;
    }
    let mut key: String = "".to_string();
    for entry in node.e.iter() {
        key = format!(
            "{}{}",
            &key[0..entry.p as usize],
            String::from_utf8_lossy(&entry.k)
        );
        map.insert(key.clone(), entry.v);
        if let Some(ref right) = entry.t {
            collect_mst_keys(db, right, map)?;
        }
    }
    Ok(())
}

// "fanout=4", meaning counting 2 bits at a time
fn leading_zeros(key: &[u8]) -> u8 {
    let digest = sha256::digest(key);
    let digest = digest.as_bytes();
    for (i, c) in digest.iter().enumerate() {
        if *c >= b'4' {
            return (i * 2) as u8;
        }
        if *c != b'0' {
            return (i * 2 + 1) as u8;
        }
    }
    digest.len() as u8
}

// # python code to generate test cases
// import hashlib
// seed = b"asdf"
// while True:
//     out = hashlib.sha256(seed).hexdigest()
//     if out.startswith("00"):
//         print(f"{seed} -> {out}")
//     seed = b"app.bsky.feed.post/" + out.encode('utf8')[:12]

#[test]
fn test_leading_zeros() {
    assert_eq!(leading_zeros(b""), 0);
    assert_eq!(leading_zeros(b"asdf"), 0);
    assert_eq!(leading_zeros(b"blue"), 1);
    assert_eq!(leading_zeros(b"2653ae71"), 0);
    assert_eq!(leading_zeros(b"88bfafc7"), 2);
    assert_eq!(leading_zeros(b"2a92d355"), 4);
    assert_eq!(leading_zeros(b"884976f5"), 6);
    assert_eq!(leading_zeros(b"app.bsky.feed.post/454397e440ec"), 4);
    assert_eq!(leading_zeros(b"app.bsky.feed.post/9adeb165882c"), 8);
}

pub fn generate_mst(
    db: &mut BlockStore<libipld::DefaultParams>,
    map: &BTreeMap<String, Cid>,
) -> Result<Cid> {
    // construct a "WIP" tree
    let mut root: Option<WipNode> = None;
    for (key, val) in map {
        let key = key.as_bytes();
        let height = leading_zeros(key);
        let entry = WipEntry {
            height,
            key: key.into(),
            val: *val,
            right: None,
        };
        if let Some(node) = root {
            root = Some(insert_entry(node, entry));
        } else {
            root = Some(WipNode {
                height: entry.height,
                left: None,
                entries: vec![entry],
            });
        }
    }
    let empty_node = WipNode {
        height: 0,
        left: None,
        entries: vec![],
    };
    serialize_wip_tree(db, root.unwrap_or(empty_node))
}

// this routine assumes that entries are added in sorted key order. AKA, the `entry` being added is
// "further right" in the tree than any existing entries
fn insert_entry(mut node: WipNode, entry: WipEntry) -> WipNode {
    // if we are higher on tree than existing node, replace it (recursively) with new layers first
    while entry.height > node.height {
        node = WipNode {
            height: node.height + 1,
            left: Some(Box::new(node)),
            entries: vec![],
        }
    }
    // if we are lower on tree, then need to descend first
    if entry.height < node.height {
        // if no entries at this node, then we should insert down "left" (which is just "down", not
        // "before" any entries)
        if node.entries.is_empty() {
            if let Some(left) = node.left {
                node.left = Some(Box::new(insert_entry(*left, entry)));
                return node;
            } else {
                panic!("hit existing totally empty MST node");
            }
        }
        let mut last = node.entries.pop().expect("hit empty existing entry list");
        assert!(entry.key > last.key);
        if last.right.is_some() {
            last.right = Some(Box::new(insert_entry(*last.right.unwrap(), entry)));
        } else {
            let mut new_node = WipNode {
                height: entry.height,
                left: None,
                entries: vec![entry],
            };
            // may need to (recursively) insert multiple filler layers
            while new_node.height + 1 < node.height {
                new_node = WipNode {
                    height: new_node.height + 1,
                    left: Some(Box::new(new_node)),
                    entries: vec![],
                }
            }
            last.right = Some(Box::new(new_node));
        }
        node.entries.push(last);
        return node;
    }
    // same height, simply append to end (but verify first)
    assert!(node.height == entry.height);
    if !node.entries.is_empty() {
        let last = &node.entries.last().unwrap();
        assert!(entry.key > last.key);
    }
    node.entries.push(entry);
    node
}

/// returns the length of common characters between the two strings. Strings must be simple ASCII,
/// which should hold for current ATP MST keys (collection plus TID)
fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    for i in 0..std::cmp::min(a.len(), b.len()) {
        if a[i] != b[i] {
            return i;
        }
    }
    // strings are the same, up to common length
    std::cmp::min(a.len(), b.len())
}

#[test]
fn test_common_prefix_len() {
    assert_eq!(common_prefix_len(b"abc", b"abc"), 3);
    assert_eq!(common_prefix_len(b"", b"abc"), 0);
    assert_eq!(common_prefix_len(b"abc", b""), 0);
    assert_eq!(common_prefix_len(b"ab", b"abc"), 2);
    assert_eq!(common_prefix_len(b"abc", b"ab"), 2);
    assert_eq!(common_prefix_len(b"abcde", b"abc"), 3);
    assert_eq!(common_prefix_len(b"abc", b"abcde"), 3);
    assert_eq!(common_prefix_len(b"abcde", b"abc1"), 3);
    assert_eq!(common_prefix_len(b"abcde", b"abb"), 2);
    assert_eq!(common_prefix_len(b"abcde", b"qbb"), 0);
    assert_eq!(common_prefix_len(b"abc", b"abc\x00"), 3);
    assert_eq!(common_prefix_len(b"abc\x00", b"abc"), 3);
}

#[test]
fn test_common_prefix_len_wide() {
    // TODO: these are not cross-language consistent!
    assert_eq!("jalapeño".as_bytes().len(), 9); // 8 in javascript
    assert_eq!("💩".as_bytes().len(), 4); // 2 in javascript
    assert_eq!("👩‍👧‍👧".as_bytes().len(), 18); // 8 in javascript

    // many of the below are different in JS; in Rust we *must* cast down to bytes to count
    assert_eq!(
        common_prefix_len("jalapeño".as_bytes(), "jalapeno".as_bytes()),
        6
    );
    assert_eq!(
        common_prefix_len("jalapeñoA".as_bytes(), "jalapeñoB".as_bytes()),
        9
    );
    assert_eq!(
        common_prefix_len("coöperative".as_bytes(), "coüperative".as_bytes()),
        3
    );
    assert_eq!(
        common_prefix_len("abc💩abc".as_bytes(), "abcabc".as_bytes()),
        3
    );
    assert_eq!(common_prefix_len("💩abc".as_bytes(), "💩ab".as_bytes()), 6);
    assert_eq!(
        common_prefix_len("abc👩‍👦‍👦de".as_bytes(), "abc👩‍👧‍👧de".as_bytes()),
        13
    );
}

fn serialize_wip_tree(
    db: &mut BlockStore<libipld::DefaultParams>,
    wip_node: WipNode,
) -> Result<Cid> {
    let left: Option<Cid> = if let Some(left) = wip_node.left {
        Some(serialize_wip_tree(db, *left)?)
    } else {
        None
    };

    let mut entries: Vec<MstEntry> = vec![];
    let mut last_key: Box<[u8]> = vec![].into();
    for wip_entry in wip_node.entries {
        let right: Option<Cid> = if let Some(right) = wip_entry.right {
            Some(serialize_wip_tree(db, *right)?)
        } else {
            None
        };
        let prefix_len = common_prefix_len(&last_key, &wip_entry.key);
        entries.push(MstEntry {
            k: wip_entry.key[prefix_len..].into(),
            p: prefix_len as u32,
            v: wip_entry.val,
            t: right,
        });
        last_key = wip_entry.key;
    }
    let mst_node = MstNode {
        l: left,
        e: entries,
    };
    let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &mst_node)?;
    let cid = *block.cid();
    db.put_block(block, None)?;
    Ok(cid)
}

#[test]
fn test_mst_node_cbor() {
    use std::str::FromStr;
    let cid1 =
        Cid::from_str("bafyreie5cvv4h45feadgeuwhbcutmh6t2ceseocckahdoe6uat64zmz454").unwrap();
    let node = MstNode {
        l: None,
        e: vec![MstEntry {
            k: "com.example.record/3jqfcqzm3fo2j".as_bytes().into(),
            p: 0,
            v: cid1,
            t: None,
        }],
    };
    let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &node).unwrap();
    println!("{block:?}");
    //assert_eq!(1, 2);
    let cid = *block.cid();
    assert_eq!(
        cid.to_string(),
        "bafyreibj4lsc3aqnrvphp5xmrnfoorvru4wynt6lwidqbm2623a6tatzdu"
    );
}
