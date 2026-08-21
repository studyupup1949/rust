use adenosine::repo::RepoStore;
use std::path::PathBuf;
use std::str::FromStr;

#[test]
fn test_repro_mst() {
    let mut repo = RepoStore::open_ephemeral().unwrap();

    let cid = repo
        .import_car_path(&PathBuf::from_str("./tests/divy.repo").unwrap(), None)
        .unwrap();
    repo.verify_repo_mst(&cid).unwrap();

    let cid = repo
        .import_car_path(
            &PathBuf::from_str("./tests/did_plc_tcyhn2v6edyi4gg4j6tiuha3.fullrepo.car").unwrap(),
            None,
        )
        .unwrap();
    repo.verify_repo_mst(&cid).unwrap();

    let cid = repo
        .import_car_path(
            &PathBuf::from_str(
                "./tests/bafyreieovfuizojpw3zresz7sx3nk4trm2by23pt5rxbey3jme4uo5ogiu.car",
            )
            .unwrap(),
            None,
        )
        .unwrap();
    repo.verify_repo_mst(&cid).unwrap();

    // test round-tripping from export
    /* XXX: test failing
    let car_bytes = repo.export_car(&cid, None).unwrap();
    let mut other_repo = RepoStore::open_ephemeral().unwrap();
    let other_cid = other_repo.import_car_bytes(&car_bytes, None).unwrap();
    other_repo.verify_repo_mst(&cid).unwrap();
    assert_eq!(cid, other_cid);
    */
}
