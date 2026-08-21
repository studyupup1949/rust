# AAD Join Info

This rust crate enables someone to get the Azure Active Directory information for a Windows computer. 

## Usage

To use `aad-join-info`, first add this to your `Cargo.toml`:

```toml
[dependencies]
aad-join-info = "0.1"
```

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(aad_info) = aad_join_info::get_aad_join_info() {
        println!("Device ID: {}", aad_info.device_id);
        println!("Tenant ID: {}", aad_info.tenant_id);
        println!("Tenant Name: {}", aad_info.tenant_name);
        println!("Idp Domain: {}", aad_info.idp_domain);
        println!("Join Type: {:?}", aad_info.join_type);
        println!("Join User Email: {}", aad_info.join_user_email);
        println!("MDM Enrollment URL: {}", aad_info.mdm_enrollment_url);
        println!("MDM Terms of Use URL: {}", aad_info.mdm_terms_of_use_url);
        println!("MDM Compliance URL: {:?}", aad_info.mdm_compliance_url);
        println!("User Setting Sync URL: {}", aad_info.user_setting_sync_url);
        println!("User Info: {:?}", aad_info.user_info);
    } else {
        println!("No AAD Join Information found.");
    }

    Ok(())
}
```