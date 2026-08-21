use adenosine::repo::RepoStore;
use libipld::Cid;
use std::collections::BTreeMap;
use std::str::FromStr;

#[test]
fn test_known_maps() {
    let mut repo = RepoStore::open_ephemeral().unwrap();
    let cid1 =
        Cid::from_str("bafyreie5cvv4h45feadgeuwhbcutmh6t2ceseocckahdoe6uat64zmz454").unwrap();

    let empty_map: BTreeMap<String, Cid> = Default::default();
    assert_eq!(
        repo.mst_from_map(&empty_map).unwrap().to_string(),
        "bafyreie5737gdxlw5i64vzichcalba3z2v5n6icifvx5xytvske7mr3hpm"
    );

    let mut trivial_map: BTreeMap<String, Cid> = Default::default();
    trivial_map.insert("com.example.record/3jqfcqzm3fo2j".to_string(), cid1);
    assert_eq!(
        repo.mst_from_map(&trivial_map).unwrap().to_string(),
        "bafyreibj4lsc3aqnrvphp5xmrnfoorvru4wynt6lwidqbm2623a6tatzdu"
    );

    let mut singlelayer2_map: BTreeMap<String, Cid> = Default::default();
    singlelayer2_map.insert("com.example.record/3jqfcqzm3fx2j".to_string(), cid1);
    assert_eq!(
        repo.mst_from_map(&singlelayer2_map).unwrap().to_string(),
        "bafyreih7wfei65pxzhauoibu3ls7jgmkju4bspy4t2ha2qdjnzqvoy33ai"
    );

    let mut simple_map: BTreeMap<String, Cid> = Default::default();
    simple_map.insert("com.example.record/3jqfcqzm3fp2j".to_string(), cid1);
    simple_map.insert("com.example.record/3jqfcqzm3fr2j".to_string(), cid1);
    simple_map.insert("com.example.record/3jqfcqzm3fs2j".to_string(), cid1);
    simple_map.insert("com.example.record/3jqfcqzm3ft2j".to_string(), cid1);
    simple_map.insert("com.example.record/3jqfcqzm4fc2j".to_string(), cid1);
    assert_eq!(
        repo.mst_from_map(&simple_map).unwrap().to_string(),
        "bafyreicmahysq4n6wfuxo522m6dpiy7z7qzym3dzs756t5n7nfdgccwq7m"
    );
}

#[test]
fn test_trims_top() {
    // "trims top of tree on delete"

    use adenosine::mst::print_mst_keys;
    let mut repo = RepoStore::open_ephemeral().unwrap();
    let cid1 =
        Cid::from_str("bafyreie5cvv4h45feadgeuwhbcutmh6t2ceseocckahdoe6uat64zmz454").unwrap();
    let l1root = "bafyreifnqrwbk6ffmyaz5qtujqrzf5qmxf7cbxvgzktl4e3gabuxbtatv4";
    let l0root = "bafyreie4kjuxbwkhzg2i5dljaswcroeih4dgiqq6pazcmunwt2byd725vi";

    // NOTE: this test doesn't do much in this case of rust implementation
    let mut trim_map: BTreeMap<String, Cid> = Default::default();
    trim_map.insert("com.example.record/3jqfcqzm3fn2j".to_string(), cid1); // level 0
    trim_map.insert("com.example.record/3jqfcqzm3fo2j".to_string(), cid1); // level 0
    trim_map.insert("com.example.record/3jqfcqzm3fp2j".to_string(), cid1); // level 0
    trim_map.insert("com.example.record/3jqfcqzm3fs2j".to_string(), cid1); // level 0
    trim_map.insert("com.example.record/3jqfcqzm3ft2j".to_string(), cid1); // level 0
    trim_map.insert("com.example.record/3jqfcqzm3fu2j".to_string(), cid1); // level 1
    let trim_before_cid = repo.mst_from_map(&trim_map).unwrap();
    print_mst_keys(&mut repo.db, &trim_before_cid).unwrap();
    assert_eq!(trim_before_cid.to_string(), l1root);

    // NOTE: if we did mutations in-place, this is where we would mutate

    trim_map.remove("com.example.record/3jqfcqzm3fs2j");
    let trim_after_cid = repo.mst_from_map(&trim_map).unwrap();
    assert_eq!(trim_after_cid.to_string(), l0root);
}

#[test]
fn test_insertion() {
    // "handles insertion that splits two layers down"

    let mut repo = RepoStore::open_ephemeral().unwrap();
    let cid1 =
        Cid::from_str("bafyreie5cvv4h45feadgeuwhbcutmh6t2ceseocckahdoe6uat64zmz454").unwrap();
    let l1root = "bafyreiettyludka6fpgp33stwxfuwhkzlur6chs4d2v4nkmq2j3ogpdjem";
    let l2root = "bafyreid2x5eqs4w4qxvc5jiwda4cien3gw2q6cshofxwnvv7iucrmfohpm";

    // TODO: actual mutation instead of rebuild from scratch
    let mut insertion_map: BTreeMap<String, Cid> = Default::default();
    insertion_map.insert("com.example.record/3jqfcqzm3fo2j".to_string(), cid1); // A; level 0
    insertion_map.insert("com.example.record/3jqfcqzm3fp2j".to_string(), cid1); // B; level 0
    insertion_map.insert("com.example.record/3jqfcqzm3fr2j".to_string(), cid1); // C; level 0
    insertion_map.insert("com.example.record/3jqfcqzm3fs2j".to_string(), cid1); // D; level 1
    insertion_map.insert("com.example.record/3jqfcqzm3ft2j".to_string(), cid1); // E; level 0
    insertion_map.insert("com.example.record/3jqfcqzm3fz2j".to_string(), cid1); // G; level 0
    insertion_map.insert("com.example.record/3jqfcqzm4fc2j".to_string(), cid1); // H; level 0
    insertion_map.insert("com.example.record/3jqfcqzm4fd2j".to_string(), cid1); // I; level 1
    insertion_map.insert("com.example.record/3jqfcqzm4ff2j".to_string(), cid1); // J; level 0
    insertion_map.insert("com.example.record/3jqfcqzm4fg2j".to_string(), cid1); // K; level 0
    insertion_map.insert("com.example.record/3jqfcqzm4fh2j".to_string(), cid1); // L; level 0
    let insertion_before_cid = repo.mst_from_map(&insertion_map).unwrap();
    assert_eq!(insertion_before_cid.to_string(), l1root);

    // insert F, which will push E out of the node with G+H to a new node under D
    insertion_map.insert("com.example.record/3jqfcqzm3fx2j".to_string(), cid1);
    let insertion_after_cid = repo.mst_from_map(&insertion_map).unwrap();
    assert_eq!(insertion_after_cid.to_string(), l2root);

    // remove F, which should push E back over with G+H
    insertion_map.remove("com.example.record/3jqfcqzm3fx2j");
    let insertion_after_cid = repo.mst_from_map(&insertion_map).unwrap();
    assert_eq!(insertion_after_cid.to_string(), l1root);
}

#[test]
fn test_higher_layers() {
    // "handles new layers that are two higher than existing"

    use adenosine::mst::print_mst_keys;
    let mut repo = RepoStore::open_ephemeral().unwrap();
    let cid1 =
        Cid::from_str("bafyreie5cvv4h45feadgeuwhbcutmh6t2ceseocckahdoe6uat64zmz454").unwrap();
    let l0root = "bafyreidfcktqnfmykz2ps3dbul35pepleq7kvv526g47xahuz3rqtptmky";
    let l2root = "bafyreiavxaxdz7o7rbvr3zg2liox2yww46t7g6hkehx4i4h3lwudly7dhy";
    let l2root2 = "bafyreig4jv3vuajbsybhyvb7gggvpwh2zszwfyttjrj6qwvcsp24h6popu";

    // TODO: actual mutation instead of rebuild from scratch
    let mut higher_map: BTreeMap<String, Cid> = Default::default();
    higher_map.insert("com.example.record/3jqfcqzm3ft2j".to_string(), cid1); // A; level 0
    higher_map.insert("com.example.record/3jqfcqzm3fz2j".to_string(), cid1); // C; level 0
    let higher_before_cid = repo.mst_from_map(&higher_map).unwrap();
    assert_eq!(higher_before_cid.to_string(), l0root);

    // insert B, which is two levels above
    higher_map.insert("com.example.record/3jqfcqzm3fx2j".to_string(), cid1); // B; level 2
    let higher_after_cid = repo.mst_from_map(&higher_map).unwrap();
    print_mst_keys(&mut repo.db, &higher_after_cid).unwrap();
    assert_eq!(higher_after_cid.to_string(), l2root);

    // remove B
    higher_map.remove("com.example.record/3jqfcqzm3fx2j");
    let higher_after_cid = repo.mst_from_map(&higher_map).unwrap();
    print_mst_keys(&mut repo.db, &higher_after_cid).unwrap();
    assert_eq!(higher_after_cid.to_string(), l0root);

    // insert B (level=2) and D (level=1)
    higher_map.insert("com.example.record/3jqfcqzm3fx2j".to_string(), cid1); // B; level 2
    higher_map.insert("com.example.record/3jqfcqzm4fd2j".to_string(), cid1); // D; level 1
    let higher_after_cid = repo.mst_from_map(&higher_map).unwrap();
    assert_eq!(higher_after_cid.to_string(), l2root2);

    // remove D
    higher_map.remove("com.example.record/3jqfcqzm4fd2j");
    let higher_after_cid = repo.mst_from_map(&higher_map).unwrap();
    assert_eq!(higher_after_cid.to_string(), l2root);
}
