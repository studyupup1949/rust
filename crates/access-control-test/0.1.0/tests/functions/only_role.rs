use crate::utils::setup::*;
use fuels::{
    prelude::*,
    types::Identity,
};

mod success {
    use super::*;

    #[tokio::test]
    async fn only_role_allows_account_with_role() {
        let (admin_wallet, account_wallet, _other_wallet, _unused_wallet) =
            setup_wallets().await;
        let (admin_instance, contract_id) =
            setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());
        let role = test_role();

        admin_instance
            .methods()
            .bootstrap_default_admin(admin)
            .call()
            .await
            .unwrap();

        admin_instance
            .methods()
            .grant_role(role, account)
            .call()
            .await
            .unwrap();

        let account_instance = AccessControlTest::new(contract_id, account_wallet);

        account_instance
            .methods()
            .only_role(role)
            .call()
            .await
            .unwrap();
    }
}

mod revert {
    use super::*;

    #[tokio::test]
    #[should_panic(expected = "AccessControlNotAuthorized")]
    async fn only_role_reverts_for_uninitialized_account() {
        let (admin_wallet, _account_wallet, _unauthorized_wallet, _unused_wallet) =
            setup_wallets().await;
        let (admin_instance, _) = setup_access_control_test(&admin_wallet).await;

        admin_instance
            .methods()
            .only_role(test_role())
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "AccessControlNotAuthorized")]
    async fn only_role_reverts_for_account_without_role() {
        let (admin_wallet, _account_wallet, unauthorized_wallet, _unused_wallet) =
            setup_wallets().await;
        let (admin_instance, contract_id) =
            setup_access_control_test(&admin_wallet).await;

        admin_instance
            .methods()
            .bootstrap_default_admin(Identity::Address(admin_wallet.address()))
            .call()
            .await
            .unwrap();

        let unauthorized_instance =
            AccessControlTest::new(contract_id, unauthorized_wallet);

        unauthorized_instance
            .methods()
            .only_role(test_role())
            .call()
            .await
            .unwrap();
    }
}
