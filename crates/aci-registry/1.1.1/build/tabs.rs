use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClassIdRegistration {
    pub id: String,
    pub internal_name: String,
    pub public_name: String,
    pub subclass_registry_owner: String,
    pub registrant: String,
    pub registrar: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct VendorIdRegistration {
    pub vendor_id: String,
    pub internal_name: String,
    pub public_name: String,
    pub product_registry_owner: String,
    pub registrant: String,
    pub registrar: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WellKnownSubclassRegistration {
    pub id: String,
    pub internal_name: String,
    pub public_name: String,
    pub specification: String,
    pub registrant: String,
    pub registrar: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OtherSubclassProductInfo {
    pub id: String,
    pub internal_name: String,
    pub public_name: String,
    pub registrant: String,
}

pub fn internal_name_to_variant(x: &str) -> String {
    let mut output = String::new();

    for group in x.split(['-', '_']) {
        let mut chars = group.chars();

        if let Some(c) = chars.next() {
            output.push(c.to_ascii_uppercase());
            output.push_str(chars.as_str())
        } else {
            continue;
        }
    }
    output
}

impl core::fmt::Display for ClassIdRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}: 0x{}, \"{}\", \"{}\", \"{}\";",
            internal_name_to_variant(&self.internal_name),
            self.id,
            self.public_name.escape_default(),
            self.subclass_registry_owner.escape_default(),
            self.registrar.escape_default()
        ))
    }
}

impl core::fmt::Display for VendorIdRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}: 0x{}, \"{}\", \"{}\", \"{}\";",
            internal_name_to_variant(&self.internal_name),
            self.vendor_id,
            self.public_name.escape_default(),
            self.product_registry_owner.escape_default(),
            self.registrar.escape_default()
        ))
    }
}

impl core::fmt::Display for WellKnownSubclassRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}: 0x{}, \"{}\", \"{}\", \"{}\";",
            internal_name_to_variant(&self.internal_name),
            self.id,
            self.public_name.escape_default(),
            self.specification.escape_default(),
            self.registrar.escape_default()
        ))
    }
}

impl core::fmt::Display for OtherSubclassProductInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}: 0x{}, \"{}\";",
            internal_name_to_variant(&self.internal_name),
            self.id,
            self.public_name.escape_default()
        ))
    }
}
