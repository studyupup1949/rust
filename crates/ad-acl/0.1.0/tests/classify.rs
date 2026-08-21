use ad_acl::{
    catalog, classify, classify_with, grants, is_dcsync, ControlPrimitive as P, SchemaMap,
};
use windows_sddl::sid::{Guid, Sid};
use windows_sddl::{AccessMask, Ace, AceType, Acl, SecurityDescriptor};

fn sid(s: &str) -> Sid {
    Sid::parse(s).expect("test SID parses")
}

fn ace(mask: AccessMask, object_type: Option<Guid>) -> Ace {
    Ace {
        ace_type: if object_type.is_some() {
            AceType::AccessAllowedObject
        } else {
            AceType::AccessAllowed
        },
        flags: 0,
        mask,
        trustee: sid("S-1-5-21-1-2-3-1105"),
        object_type,
        inherited_object_type: None,
    }
}

#[test]
fn generic_all_is_full_control() {
    assert_eq!(
        classify(&ace(AccessMask::GENERIC_ALL, None)),
        vec![P::GenericAll]
    );
}

#[test]
fn write_prop_without_guid_is_generic_write() {
    assert_eq!(
        classify(&ace(AccessMask::WRITE_PROP, None)),
        vec![P::GenericWrite]
    );
}

#[test]
fn key_credential_link_is_shadow_credentials() {
    let a = ace(
        AccessMask::WRITE_PROP,
        Some(catalog::KEY_CREDENTIAL_LINK.guid()),
    );
    assert_eq!(classify(&a), vec![P::AddKeyCredential]);
}

#[test]
fn rbcd_and_spn_and_gplink_writes() {
    for (guid, want) in [
        (catalog::RBCD.guid(), P::WriteRbcd),
        (catalog::SPN.guid(), P::WriteSpn),
        (catalog::GP_LINK.guid(), P::WriteGpLink),
        (catalog::MEMBER.guid(), P::AddMember),
        (
            catalog::ALT_SECURITY_IDENTITIES.guid(),
            P::WriteAltSecurityIdentities,
        ),
        (
            catalog::ALLOWED_TO_DELEGATE_TO.guid(),
            P::WriteAllowedToDelegateTo,
        ),
    ] {
        assert_eq!(
            classify(&ace(AccessMask::WRITE_PROP, Some(guid))),
            vec![want]
        );
    }
}

#[test]
fn unscoped_control_access_is_every_extended_right() {
    assert_eq!(
        classify(&ace(AccessMask::CONTROL_ACCESS, None)),
        vec![P::AllExtendedRights]
    );
}

#[test]
fn replication_rights_split_into_halves() {
    assert_eq!(
        classify(&ace(
            AccessMask::CONTROL_ACCESS,
            Some(catalog::REPL_GET_CHANGES.guid())
        )),
        vec![P::DcsyncGetChanges]
    );
    assert_eq!(
        classify(&ace(
            AccessMask::CONTROL_ACCESS,
            Some(catalog::REPL_GET_CHANGES_ALL.guid())
        )),
        vec![P::DcsyncGetChangesAll]
    );
}

#[test]
fn dcsync_needs_the_secret_bearing_half() {
    assert!(!is_dcsync(&[P::DcsyncGetChanges]));
    assert!(is_dcsync(&[P::DcsyncGetChanges, P::DcsyncGetChangesAll]));
    assert!(is_dcsync(&[P::AllExtendedRights]));
    assert!(is_dcsync(&[P::GenericAll]));
}

#[test]
fn validated_writes_are_self_scoped() {
    assert_eq!(
        classify(&ace(AccessMask::SELF, Some(catalog::MEMBER.guid()))),
        vec![P::AddSelfToGroup]
    );
    assert_eq!(
        classify(&ace(AccessMask::SELF, Some(catalog::SPN.guid()))),
        vec![P::WriteSpn]
    );
}

#[test]
fn deny_ace_grants_nothing() {
    let mut a = ace(AccessMask::GENERIC_ALL, None);
    a.ace_type = AceType::AccessDenied;
    assert!(classify(&a).is_empty());
}

#[test]
fn unknown_object_guid_is_not_a_primitive() {
    let unknown = Guid::parse("11111111-2222-3333-4444-555555555555").unwrap();
    assert!(classify(&ace(AccessMask::WRITE_PROP, Some(unknown))).is_empty());
    assert!(classify(&ace(AccessMask::CONTROL_ACCESS, Some(unknown))).is_empty());
}

#[test]
fn read_prop_needs_the_schema_map() {
    let laps = Guid::parse("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap();
    let a = ace(AccessMask::READ_PROP, Some(laps));
    // Without the forest schema this GUID means nothing.
    assert!(classify(&a).is_empty());

    let schema = SchemaMap::from_entries([(ad_acl::names::LAPS_ENCRYPTED, laps)]);
    assert_eq!(classify_with(&a, &schema), vec![P::ReadLapsPassword]);
}

#[test]
fn create_child_scoped_to_dmsa_is_badsuccessor() {
    let dmsa = Guid::parse("99999999-8888-7777-6666-555555555555").unwrap();
    let a = ace(AccessMask::CREATE_CHILD, Some(dmsa));
    assert_eq!(classify(&a), vec![P::CreateChild(Some(dmsa))]);

    let schema = SchemaMap::from_entries([(ad_acl::names::DMSA_CLASS, dmsa)]);
    assert_eq!(classify_with(&a, &schema), vec![P::CreateDmsa]);
}

#[test]
fn grants_include_the_owner() {
    let sd = SecurityDescriptor {
        owner: Some(sid("S-1-5-21-1-2-3-500")),
        group: None,
        dacl: Some(Acl {
            aces: vec![ace(AccessMask::WRITE_DAC, None)],
        }),
    };
    let g = grants(&sd);
    assert_eq!(g.len(), 2);
    assert_eq!(g[0].primitive, P::Owns);
    assert_eq!(g[1].primitive, P::WriteDacl);
    assert!(!g[1].inherited);
}

#[test]
fn every_primitive_has_impact_and_mitigation() {
    for p in [
        P::Owns,
        P::WriteDacl,
        P::GenericAll,
        P::AddKeyCredential,
        P::WriteRbcd,
        P::ReadLapsPassword,
        P::ReadGmsaPassword,
        P::CreateDmsa,
        P::DcsyncGetChangesAll,
        P::Enroll,
    ] {
        assert!(!p.name().is_empty());
        assert!(!p.impact().is_empty());
        assert!(!p.mitigation().is_empty());
        assert!(p.cost() <= 3);
    }
}
