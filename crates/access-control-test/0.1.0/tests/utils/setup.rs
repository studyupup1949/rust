use fuels::{
    accounts::{
        signers::private_key::PrivateKeySigner,
        wallet::Unlocked,
    },
    prelude::*,
    types::ContractId,
};

abigen!(Contract(
    name = "AccessControlTest",
    abi = "contracts/test_artifacts/access_control_test/out/debug/access_control_test-abi.json"
));

pub async fn setup_wallets() -> (
    Wallet<Unlocked<PrivateKeySigner>>,
    Wallet<Unlocked<PrivateKeySigner>>,
    Wallet<Unlocked<PrivateKeySigner>>,
    Wallet<Unlocked<PrivateKeySigner>>,
) {
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(4), Some(1), Some(1_000_000_000)),
        None,
        Some(::fuels::test_helpers::ChainConfig::local_testnet()),
    )
    .await
    .unwrap();

    let wallet_1 = wallets.pop().unwrap();
    let wallet_2 = wallets.pop().unwrap();
    let wallet_3 = wallets.pop().unwrap();
    let wallet_4 = wallets.pop().unwrap();

    (wallet_1, wallet_2, wallet_3, wallet_4)
}

pub async fn setup_access_control_test(
    deploy_wallet: &Wallet<Unlocked<PrivateKeySigner>>,
) -> (
    AccessControlTest<Wallet<Unlocked<PrivateKeySigner>>>,
    ContractId,
) {
    let id = Contract::load_from(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/out/debug/access_control_test.bin"
        ),
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(deploy_wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let instance = AccessControlTest::new(id, deploy_wallet.clone());

    (instance, id)
}

pub fn admin_role() -> u64 {
    0
}

pub fn test_role() -> u64 {
    1
}

pub fn other_role() -> u64 {
    2
}

pub fn last_role() -> u64 {
    63
}

pub fn role_bitmap(roles: &[u64]) -> u64 {
    roles.iter().fold(0, |bitmap, role| bitmap | (1u64 << role))
}
