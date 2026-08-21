//! GENERATED FILE - DO NOT EDIT
//! Source: protocol/transaction.yml
//! Generated: 2025-10-03 22:05:19

#![allow(missing_docs)]

use serde::{Serialize, Deserialize};


mod hex_option_vec {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => serializer.serialize_str(&hex::encode(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(hex_str) => {
                hex::decode(&hex_str).map(Some).map_err(D::Error::custom)
            }
            None => Ok(None),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpireOptions {
    pub at_time: Option<u64>,
}

impl ExpireOptions {
    pub fn validate(&self) -> Result<(), crate::errors::Error> {
        // If at_time is provided, it should be a reasonable timestamp
        // (not in the distant past, and not impossibly far in the future)
        if let Some(at_time) = self.at_time {
            // Timestamps before year 2000 are likely errors (Unix timestamp 946684800)
            const YEAR_2000_UNIX: u64 = 946684800;
            // Timestamps more than 100 years in the future are likely errors
            const HUNDRED_YEARS_SECONDS: u64 = 100 * 365 * 24 * 60 * 60;

            if at_time > 0 && at_time < YEAR_2000_UNIX {
                return Err(crate::errors::ValidationError::OutOfRange {
                    field: "atTime".to_string(),
                    min: YEAR_2000_UNIX.to_string(),
                    max: "far future".to_string(),
                }.into());
            }

            // Get current time estimate (we can't use std::time here due to no_std compatibility concerns,
            // but we can at least check for obviously invalid future values)
            if at_time > YEAR_2000_UNIX + HUNDRED_YEARS_SECONDS {
                return Err(crate::errors::ValidationError::OutOfRange {
                    field: "atTime".to_string(),
                    min: "now".to_string(),
                    max: "100 years from epoch".to_string(),
                }.into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldUntilOptions {
    pub minor_block: Option<u64>,
}

impl HoldUntilOptions {
    pub fn validate(&self) -> Result<(), crate::errors::Error> {
        // If minor_block is provided, it should be positive (block numbers start at 1)
        if let Some(minor_block) = self.minor_block {
            if minor_block == 0 {
                return Err(crate::errors::ValidationError::InvalidFieldValue {
                    field: "minorBlock".to_string(),
                    reason: "minor block number must be greater than zero".to_string(),
                }.into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHeader {
    pub principal: String,
    #[serde(with = "hex::serde")]
    pub initiator: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[serde(with = "hex_option_vec")]
    pub metadata: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expire: Option<ExpireOptions>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hold_until: Option<HoldUntilOptions>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub authorities: Option<Vec<String>>,
}

impl TransactionHeader {
    /// Field-level validation aligned with YAML truth
    pub fn validate(&self) -> Result<(), crate::errors::Error> {
        if self.principal.is_empty() { return Err(crate::errors::Error::General("Principal URL cannot be empty".to_string())); }

        // Validate principal URL contains only ASCII characters
        if !self.principal.is_ascii() {
            return Err(crate::errors::Error::General("Principal URL must contain only ASCII characters".to_string()));
        }

        // Validate initiator size (reasonable limit: 32KB)
        const MAX_INITIATOR_SIZE: usize = 32 * 1024;
        if self.initiator.len() > MAX_INITIATOR_SIZE {
            return Err(crate::errors::Error::General(format!("Initiator size {} exceeds maximum of {}", self.initiator.len(), MAX_INITIATOR_SIZE)));
        }

        // Validate authorities (Additional Authorities)
        if let Some(ref authorities) = self.authorities {
            self.validate_authorities(authorities)?;
        }

        // Validate metadata - no null bytes allowed in binary metadata
        if let Some(ref metadata) = self.metadata {
            if metadata.contains(&0) {
                return Err(crate::errors::Error::General("Metadata cannot contain null bytes".to_string()));
            }
        }

        if let Some(ref opts) = self.expire { opts.validate()?; }
        if let Some(ref opts) = self.hold_until { opts.validate()?; }
        Ok(())
    }

    /// Validate additional authorities list
    fn validate_authorities(&self, authorities: &[String]) -> Result<(), crate::errors::Error> {
        // Maximum number of additional authorities per Go protocol limits
        const MAX_AUTHORITIES: usize = 20;

        if authorities.len() > MAX_AUTHORITIES {
            return Err(crate::errors::ValidationError::InvalidFieldValue {
                field: "authorities".to_string(),
                reason: format!("too many authorities: {} (max {})", authorities.len(), MAX_AUTHORITIES),
            }.into());
        }

        for (index, authority) in authorities.iter().enumerate() {
            // Authority URL cannot be empty
            if authority.is_empty() {
                return Err(crate::errors::ValidationError::InvalidFieldValue {
                    field: format!("authorities[{}]", index),
                    reason: "authority URL cannot be empty".to_string(),
                }.into());
            }

            // Authority URL must start with acc://
            if !authority.starts_with("acc://") {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: must start with 'acc://', got '{}'", index, authority)
                ).into());
            }

            // Authority URL must contain only ASCII characters
            if !authority.is_ascii() {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: URL must contain only ASCII characters", index)
                ).into());
            }

            // Authority URL must not contain whitespace
            if authority.chars().any(|c| c.is_whitespace()) {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: URL must not contain whitespace", index)
                ).into());
            }

            // Check for reasonable URL length
            const MAX_URL_LENGTH: usize = 1024;
            if authority.len() > MAX_URL_LENGTH {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: URL too long ({} > {})", index, authority.len(), MAX_URL_LENGTH)
                ).into());
            }

            // Authority should point to a key book (must contain /book/ or /page/ pattern)
            // This is advisory - some authorities may be identities themselves
            // Relaxed validation: just ensure it's a valid Accumulate URL structure
            let url_path = &authority[6..]; // Skip "acc://"
            if url_path.is_empty() || url_path == "/" {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: URL has no identity", index)
                ).into());
            }
        }

        // Check for duplicate authorities
        let mut seen = std::collections::HashSet::new();
        for (index, authority) in authorities.iter().enumerate() {
            let normalized = authority.to_lowercase();
            if !seen.insert(normalized.clone()) {
                return Err(crate::errors::ValidationError::InvalidFieldValue {
                    field: format!("authorities[{}]", index),
                    reason: format!("duplicate authority URL: {}", authority),
                }.into());
            }
        }

        Ok(())
    }
}