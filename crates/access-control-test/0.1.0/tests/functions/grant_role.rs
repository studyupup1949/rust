use crate::utils::setup::*;
use fuels::{
    prelude::*,
    types::Identity,
};

mod success {
    use super::*;

    #[tokio::test]
    async fn grant_and_revoke_role() {
        let (admin_wallet, account_wallet, _other_wallet, _unused_wallet) =
            setup_wallets().await;
        let (instance, contract_id) = setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());
        let role = test_role();

        assert_eq!(
            instance
                .methods()
                .default_admin_role()
                .call()
                .await
                .unwrap()
                .value,
            0
        );
        assert!(
            !instance
                .methods()
                .has_role(role, account)
                .call()
                .await
                .unwrap()
                .value
        );

        instance
            .methods()
            .bootstrap_default_admin(admin)
            .call()
            .await
            .unwrap();

        let grant_response = instance
            .methods()
            .grant_role(role, account)
            .call()
            .await
            .unwrap();
        let grant_logs = grant_response
            .decode_logs_with_type::<RoleGrantedEvent>()
            .unwrap();
        assert_eq!(grant_logs, vec![RoleGrantedEvent { role, account }]);

        assert!(
            instance
                .methods()
                .has_role(role, account)
                .call()
                .await
                .unwrap()
                .value
        );

        let account_instance = AccessControlTest::new(contract_id, account_wallet);

        account_instance
            .methods()
            .only_role(role)
            .call()
            .await
            .unwrap();

        let revoke_response = instance
            .methods()
            .revoke_role(role, account)
            .call()
            .await
            .unwrap();
        let revoke_logs = revoke_response
            .decode_logs_with_type::<RoleRevokedEvent>()
            .unwrap();
        assert_eq!(revoke_logs, vec![RoleRevokedEvent { role, account }]);

        assert!(
            !instance
                .methods()
                .has_role(role, account)
                .call()
                .await
                .unwrap()
                .value
        );

        let post_revoke_call = account_instance.methods().only_role(role).call().await;

        assert!(post_revoke_call.is_err());
    }

    #[tokio::test]
    async fn granting_role_does_not_grant_different_role_to_same_user() {
        let (admin_wallet, account_wallet, _other_wallet, _unused_wallet) =
            setup_wallets().await;
        let (instance, _id) = setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());

        instance
            .methods()
            .bootstrap_default_admin(admin)
            .call()
            .await
            .unwrap();

        instance
            .methods()
            .grant_role(test_role(), account)
            .call()
            .await
            .unwrap();

        assert!(
            instance
                .methods()
                .has_role(test_role(), account)
                .call()
                .await
                .unwrap()
                .value
        );
        assert!(
            !instance
                .methods()
                .has_role(other_role(), account)
                .call()
                .await
                .unwrap()
                .value
        );
    }

    #[tokio::test]
    async fn granting_role_does_not_grant_same_role_to_different_user() {
        let (admin_wallet, account_wallet, other_wallet, _unused_wallet) =
            setup_wallets().await;
        let (instance, _id) = setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());
        let other_account = Identity::Address(other_wallet.address());
        let role = test_role();

        instance
            .methods()
            .bootstrap_default_admin(admin)
            .call()
            .await
            .unwrap();

        instance
            .methods()
            .grant_role(role, account)
            .call()
            .await
            .unwrap();

        assert!(
            instance
                .methods()
                .has_role(role, account)
                .call()
                .await
                .unwrap()
                .value
        );
        assert!(
            !instance
                .methods()
                .has_role(role, other_account)
                .call()
                .await
                .unwrap()
                .value
        );
    }

    #[tokio::test]
    async fn granting_roles_updates_bitmap_with_expected_bits() {
        let (admin_wallet, account_wallet, _other_wallet, _unused_wallet) =
            setup_wallets().await;
        let (instance, _id) = setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());
        let role_admin = admin_role();
        let role_test = test_role();
        let role_other = other_role();
        let role_last = last_role();
        let ungranted_role = 3;

        assert_eq!(
            instance
                .methods()
                .get_roles(admin)
                .call()
                .await
                .unwrap()
                .value,
            0
        );
        assert_eq!(
            instance
                .methods()
                .get_roles(account)
                .call()
                .await
                .unwrap()
                .value,
            0
        );

        instance
            .methods()
            .bootstrap_default_admin(admin)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .methods()
                .get_roles(admin)
                .call()
                .await
                .unwrap()
                .value,
            role_bitmap(&[role_admin])
        );

        instance
            .methods()
            .grant_role(role_test, account)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .methods()
                .get_roles(account)
                .call()
                .await
                .unwrap()
                .value,
            role_bitmap(&[role_test])
        );

        instance
            .methods()
            .grant_role(role_other, account)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .methods()
                .get_roles(account)
                .call()
                .await
                .unwrap()
                .value,
            role_bitmap(&[role_test, role_other])
        );

        instance
            .methods()
            .grant_role(role_admin, account)
            .call()
            .await
            .unwrap();

        instance
            .methods()
            .grant_role(role_last, account)
            .call()
            .await
            .unwrap();

        assert_eq!(
            instance
                .methods()
                .get_roles(account)
                .call()
                .await
                .unwrap()
                .value,
            role_bitmap(&[role_admin, role_test, role_other, role_last])
        );

        for role in [role_admin, role_test, role_other, role_last] {
            assert!(
                instance
                    .methods()
                    .has_role(role, account)
                    .call()
                    .await
                    .unwrap()
                    .value
            );
        }

        assert!(
            !instance
                .methods()
                .has_role(ungranted_role, account)
                .call()
                .await
                .unwrap()
                .value
        );
    }

    #[tokio::test]
    async fn granted_admin_can_grant_roles_including_admin_role() {
        let (root_admin_wallet, delegated_admin_wallet, user_wallet, other_wallet) =
            setup_wallets().await;
        let (root_admin_instance, contract_id) =
            setup_access_control_test(&root_admin_wallet).await;

        let root_admin = Identity::Address(root_admin_wallet.address());
        let delegated_admin = Identity::Address(delegated_admin_wallet.address());
        let user = Identity::Address(user_wallet.address());
        let other_user = Identity::Address(other_wallet.address());
        let admin_role = 0;
        let role = test_role();

        root_admin_instance
            .methods()
            .bootstrap_default_admin(root_admin)
            .call()
            .await
            .unwrap();

        root_admin_instance
            .methods()
            .grant_role(admin_role, delegated_admin)
            .call()
            .await
            .unwrap();

        let delegated_admin_instance =
            AccessControlTest::new(contract_id, delegated_admin_wallet);

        delegated_admin_instance
            .methods()
            .grant_role(role, user)
            .call()
            .await
            .unwrap();

        delegated_admin_instance
            .methods()
            .grant_role(admin_role, other_user)
            .call()
            .await
            .unwrap();

        assert!(
            root_admin_instance
                .methods()
                .has_role(role, user)
                .call()
                .await
                .unwrap()
                .value
        );
        assert!(
            root_admin_instance
                .methods()
                .has_role(admin_role, other_user)
                .call()
                .await
                .unwrap()
                .value
        );
    }

    #[tokio::test]
    async fn granted_admin_can_revoke_roles_including_admin_role() {
        let (root_admin_wallet, delegated_admin_wallet, user_wallet, other_wallet) =
            setup_wallets().await;
        let (root_admin_instance, contract_id) =
            setup_access_control_test(&root_admin_wallet).await;

        let root_admin = Identity::Address(root_admin_wallet.address());
        let delegated_admin = Identity::Address(delegated_admin_wallet.address());
        let user = Identity::Address(user_wallet.address());
        let other_admin = Identity::Address(other_wallet.address());
        let admin_role = 0;
        let role = test_role();

        root_admin_instance
            .methods()
            .bootstrap_default_admin(root_admin)
            .call()
            .await
            .unwrap();

        root_admin_instance
            .methods()
            .grant_role(admin_role, delegated_admin)
            .call()
            .await
            .unwrap();

        root_admin_instance
            .methods()
            .grant_role(role, user)
            .call()
            .await
            .unwrap();

        root_admin_instance
            .methods()
            .grant_role(admin_role, other_admin)
            .call()
            .await
            .unwrap();

        let delegated_admin_instance =
            AccessControlTest::new(contract_id, delegated_admin_wallet);
        let user_instance = AccessControlTest::new(contract_id, user_wallet);
        let other_admin_instance = AccessControlTest::new(contract_id, other_wallet);

        user_instance
            .methods()
            .only_role(role)
            .call()
            .await
            .unwrap();
        other_admin_instance
            .methods()
            .only_role(admin_role)
            .call()
            .await
            .unwrap();

        delegated_admin_instance
            .methods()
            .revoke_role(role, user)
            .call()
            .await
            .unwrap();

        delegated_admin_instance
            .methods()
            .revoke_role(admin_role, other_admin)
            .call()
            .await
            .unwrap();

        assert!(
            !root_admin_instance
                .methods()
                .has_role(role, user)
                .call()
                .await
                .unwrap()
                .value
        );
        assert!(
            !root_admin_instance
                .methods()
                .has_role(admin_role, other_admin)
                .call()
                .await
                .unwrap()
                .value
        );

        let revoked_user_call = user_instance.methods().only_role(role).call().await;
        let revoked_admin_call = other_admin_instance
            .methods()
            .only_role(admin_role)
            .call()
            .await;

        assert!(revoked_user_call.is_err());
        assert!(revoked_admin_call.is_err());
    }
}

mod revert {
    use super::*;

    #[tokio::test]
    #[should_panic(expected = "AccessControlNotAuthorized")]
    async fn when_grant_role_caller_is_not_admin() {
        let (admin_wallet, account_wallet, unauthorized_wallet, _unused_wallet) =
            setup_wallets().await;
        let (admin_instance, contract_id) =
            setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());

        admin_instance
            .methods()
            .bootstrap_default_admin(admin)
            .call()
            .await
            .unwrap();

        let unauthorized_instance =
            AccessControlTest::new(contract_id, unauthorized_wallet);

        unauthorized_instance
            .methods()
            .grant_role(test_role(), account)
            .call()
            .await
            .unwrap();
    }
}
