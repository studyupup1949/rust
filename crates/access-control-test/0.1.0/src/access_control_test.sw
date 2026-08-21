contract;

use contract_libs::access_control::{
    ACCESS_CONTROL_SLOT,
    DEFAULT_ADMIN_ROLE,
    Role,
    RoleBitmap,
    unchecked_grant_role as access_control_unchecked_grant_role,
    grant_role as access_control_grant_role,
    get_roles as access_control_get_roles,
    has_role as access_control_has_role,
    only_role as access_control_only_role,
    revoke_role as access_control_revoke_role,
};

abi AccessControlTest {
    fn access_control_slot() -> b256;

    fn default_admin_role() -> Role;

    #[storage(read)]
    fn get_roles(account: Identity) -> RoleBitmap;

    #[storage(read)]
    fn has_role(role: Role, account: Identity) -> bool;

    #[storage(read)]
    fn only_role(role: Role);

    #[storage(read, write)]
    fn grant_role(role: Role, account: Identity);

    #[storage(read, write)]
    fn revoke_role(role: Role, account: Identity);

    #[storage(read, write)]
    fn bootstrap_default_admin(account: Identity);
}

impl AccessControlTest for Contract {
    fn access_control_slot() -> b256 {
        ACCESS_CONTROL_SLOT
    }

    fn default_admin_role() -> Role {
        DEFAULT_ADMIN_ROLE
    }

    #[storage(read)]
    fn get_roles(account: Identity) -> RoleBitmap {
        access_control_get_roles(account)
    }

    #[storage(read)]
    fn has_role(role: Role, account: Identity) -> bool {
        access_control_has_role(role, account)
    }

    #[storage(read)]
    fn only_role(role: Role) {
        access_control_only_role(role);
    }

    #[storage(read, write)]
    fn grant_role(role: Role, account: Identity) {
        access_control_grant_role(role, account);
    }

    #[storage(read, write)]
    fn revoke_role(role: Role, account: Identity) {
        access_control_revoke_role(role, account);
    }

    #[storage(read, write)]
    fn bootstrap_default_admin(account: Identity) {
        access_control_unchecked_grant_role(DEFAULT_ADMIN_ROLE, account);
    }
}
