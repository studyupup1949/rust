#[cfg(target_os = "windows")]
use windows::{Win32::{
    NetworkManagement::NetManagement::{
        NetGetAadJoinInformation, NetFreeAadJoinInformation, DSREG_JOIN_INFO, DSREG_USER_INFO, DSREG_JOIN_TYPE
    }
}, core::{PCWSTR}};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AADJoinInformationUserInfo {
    pub user_key_name: String,
    pub user_email: String,
    pub user_key_id: String,
}

#[cfg(target_os = "windows")]
impl From<*mut DSREG_USER_INFO> for AADJoinInformationUserInfo {
    fn from(value: *mut DSREG_USER_INFO) -> Self {
        unsafe {
            let user_info = *value;
            AADJoinInformationUserInfo {
                user_email: user_info.pszUserEmail.to_string().unwrap(),
                user_key_id: user_info.pszUserKeyId.to_string().unwrap(),
                user_key_name: user_info.pszUserKeyName.to_string().unwrap(),
            }
        }
    }    
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AADJoinInformationJoinType {
    Unknown = 0,
    DeviceJoin = 1,
    WorkplaceJoin = 2,
}

#[cfg(target_os = "windows")]
impl From<DSREG_JOIN_TYPE> for AADJoinInformationJoinType {
    fn from(value: DSREG_JOIN_TYPE) -> Self {
        match value {
            DSREG_JOIN_TYPE(0i32) => AADJoinInformationJoinType::Unknown,
            DSREG_JOIN_TYPE(1i32) => AADJoinInformationJoinType::DeviceJoin,
            DSREG_JOIN_TYPE(3i32) => AADJoinInformationJoinType::WorkplaceJoin,
            _ => AADJoinInformationJoinType::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AADJoinInformation {
    pub join_type: AADJoinInformationJoinType,
    pub tenant_id: String,
    pub tenant_name: String,
    pub device_id: String,
    pub idp_domain: String,
    pub join_user_email: String,
    pub mdm_enrollment_url: String,
    pub mdm_terms_of_use_url: String,
    pub mdm_compliance_url: String,
    pub user_setting_sync_url: String,
    pub user_info: Option<AADJoinInformationUserInfo>,
}

#[cfg(not(target_os = "windows"))]
pub fn get_aad_join_info() -> Option<AADJoinInformation> {
    None
}

#[cfg(target_os = "windows")]
pub fn get_aad_join_info() -> Option<AADJoinInformation> {
    unsafe { get_aad_join_info_unsafe() }
}

#[cfg(target_os = "windows")]
unsafe fn get_aad_join_info_unsafe() -> Option<AADJoinInformation> {
    const SESSION_ID: PCWSTR = PCWSTR(std::ptr::null_mut());


    let join_info: Option<*const DSREG_JOIN_INFO> = match NetGetAadJoinInformation(SESSION_ID) {
        Ok(info) => Some(info),
        Err(_) => { None },
    };

    let aad_info = if let Some(info) = join_info {    
        let dsreg_info = *info;            
        Some(AADJoinInformation {
            device_id: dsreg_info.pszDeviceId.to_string().unwrap(),
            idp_domain: dsreg_info.pszIdpDomain.to_string().unwrap(),
            join_type: AADJoinInformationJoinType::from(dsreg_info.joinType),
            join_user_email: dsreg_info.pszJoinUserEmail.to_string().unwrap(),
            mdm_compliance_url: dsreg_info.pszMdmComplianceUrl.to_string().unwrap(),
            mdm_enrollment_url: dsreg_info.pszMdmEnrollmentUrl.to_string().unwrap(),
            mdm_terms_of_use_url: dsreg_info.pszMdmTermsOfUseUrl.to_string().unwrap(),
            user_setting_sync_url: dsreg_info.pszUserSettingSyncUrl.to_string().unwrap(),
            tenant_id: dsreg_info.pszTenantId.to_string().unwrap(),
            tenant_name: dsreg_info.pszTenantDisplayName.to_string().unwrap(),
            user_info: Some(AADJoinInformationUserInfo::from(dsreg_info.pUserInfo))            
        })
    }
    else {
        None
    };

    NetFreeAadJoinInformation(join_info);
    aad_info
}