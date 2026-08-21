use crate::utils::setup::*;
use fuels::{
    prelude::*,
    types::Identity,
};

mod success {
    use super::*;

    #[tokio::test]
    async fn revoke_role_does_not_revoke_unrelated_role() {
        let (admin_wallet, account_wallet, _other_wallet, _unused_wallet) =
            setup_wallets().await;
        let (instance, contract_id) = setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());
        let role_admin = admin_role();
        let role_test = test_role();
        let role_other = other_role();

        assert!(
            !instance
                .methods()
                .has_role(role_admin, account)
                .call()
                .await
                .unwrap()
                .value
        );

        assert!(
            !instance
                .methods()
                .has_role(role_test, account)
                .call()
                .await
                .unwrap()
                .value
        );

        assert!(
            !instance
                .methods()
                .has_role(role_other, account)
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

        let _ = instance
            .methods()
            .grant_role(role_admin, account)
            .call()
            .await
            .unwrap();

        let _ = instance
            .methods()
            .grant_role(role_test, account)
            .call()
            .await
            .unwrap();

        let _ = instance
            .methods()
            .grant_role(role_other, account)
            .call()
            .await
            .unwrap();

        assert!(
            instance
                .methods()
                .has_role(role_admin, account)
                .call()
                .await
                .unwrap()
                .value
        );

        assert!(
            instance
                .methods()
                .has_role(role_test, account)
                .call()
                .await
                .unwrap()
                .value
        );

        assert!(
            instance
                .methods()
                .has_role(role_other, account)
                .call()
                .await
                .unwrap()
                .value
        );

        let revoke_response = instance
            .methods()
            .revoke_role(role_test, account)
            .call()
            .await
            .unwrap();
        let revoke_logs = revoke_response
            .decode_logs_with_type::<RoleRevokedEvent>()
            .unwrap();
        assert_eq!(
            revoke_logs,
            vec![RoleRevokedEvent {
                role: role_test,
                account,
            }]
        );

        assert!(
            !instance
                .methods()
                .has_role(role_test, account)
                .call()
                .await
                .unwrap()
                .value
        );

        assert!(
            instance
                .methods()
                .has_role(role_admin, account)
                .call()
                .await
                .unwrap()
                .value
        );

        assert!(
            instance
                .methods()
                .has_role(role_other, account)
                .call()
                .await
                .unwrap()
                .value
        );

        let account_instance = AccessControlTest::new(contract_id, account_wallet);
        let post_revoke_call_admin = account_instance
            .methods()
            .only_role(role_admin)
            .call()
            .await;
        assert!(post_revoke_call_admin.is_ok());

        let post_revoke_call_other = account_instance
            .methods()
            .only_role(role_other)
            .call()
            .await;
        assert!(post_revoke_call_other.is_ok());
    }

    #[tokio::test]
    async fn revoking_roles_updates_bitmap_without_clearing_unrelated_bits() {
        let (admin_wallet, account_wallet, _other_wallet, _unused_wallet) =
            setup_wallets().await;
        let (instance, _id) = setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());
        let role_admin = admin_role();
        let role_test = test_role();
        let role_other = other_role();
        let role_last = last_role();

        instance
            .methods()
            .bootstrap_default_admin(admin)
            .call()
            .await
            .unwrap();

        for role in [role_admin, role_test, role_other, role_last] {
            instance
                .methods()
                .grant_role(role, account)
                .call()
                .await
                .unwrap();
        }

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

        instance
            .methods()
            .revoke_role(role_test, account)
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
            role_bitmap(&[role_admin, role_other, role_last])
        );
        assert!(
            !instance
                .methods()
                .has_role(role_test, account)
                .call()
                .await
                .unwrap()
                .value
        );
        assert!(
            instance
                .methods()
                .has_role(role_admin, account)
                .call()
                .await
                .unwrap()
                .value
        );
        assert!(
            instance
                .methods()
                .has_role(role_other, account)
                .call()
                .await
                .unwrap()
                .value
        );
        assert!(
            instance
                .methods()
                .has_role(role_last, account)
                .call()
                .await
                .unwrap()
                .value
        );

        instance
            .methods()
            .revoke_role(role_last, account)
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
            role_bitmap(&[role_admin, role_other])
        );

        instance
            .methods()
            .revoke_role(role_other, account)
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
            role_bitmap(&[role_admin])
        );

        instance
            .methods()
            .revoke_role(role_admin, account)
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
            0
        );

        for role in [role_admin, role_test, role_other, role_last] {
            assert!(
                !instance
                    .methods()
                    .has_role(role, account)
                    .call()
                    .await
                    .unwrap()
                    .value
            );
        }
    }
}

mod revert {
    use super::*;

    #[tokio::test]
    #[should_panic(expected = "AccessControlNotAuthorized")]
    async fn only_role_reverts_after_role_revoked() {
        let (admin_wallet, account_wallet, _other_wallet, _unused_wallet) =
            setup_wallets().await;
        let (instance, contract_id) = setup_access_control_test(&admin_wallet).await;

        let admin = Identity::Address(admin_wallet.address());
        let account = Identity::Address(account_wallet.address());
        let role_test = test_role();

        instance
            .methods()
            .bootstrap_default_admin(admin)
            .call()
            .await
            .unwrap();

        let _ = instance
            .methods()
            .grant_role(role_test, account)
            .call()
            .await
            .unwrap();

        assert!(
            instance
                .methods()
                .has_role(role_test, account)
                .call()
                .await
                .unwrap()
                .value
        );

        let _ = instance
            .methods()
            .revoke_role(role_test, account)
            .call()
            .await
            .unwrap();

        assert!(
            !instance
                .methods()
                .has_role(role_test, account)
                .call()
                .await
                .unwrap()
                .value
        );

        let account_instance = AccessControlTest::new(contract_id, account_wallet);
        account_instance
            .methods()
            .only_role(role_test)
            .call()
            .await
            .unwrap();
    }
}
