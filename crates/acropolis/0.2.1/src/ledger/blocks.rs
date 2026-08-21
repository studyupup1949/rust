use crate::chain::{ChainPoint, ChainTip};
use crate::events::{Event, EventPayload};
use std::fmt;

pub const BLOCK_ERA_VALIDATION_EVENT: &str = "ledger.block_era_validation";
pub const BLOCK_ERA_VALIDATION_FAILED_EVENT: &str = "ledger.block_era_validation_failed";
pub const BLOCK_DECODE_FAILED_EVENT: &str = "ledger.block_decode_failed";

const LOCAL_BLOCK_MAGIC: &[u8; 8] = b"ACBLK001";
const MAX_DECODED_ITEMS: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub point: ChainPoint,
    pub bundle_number: u64,
    pub parent_hash: [u8; 32],
    pub body_hash: [u8; 32],
}

impl BlockHeader {
    pub fn tip(&self) -> ChainTip {
        ChainTip::new(self.point, self.bundle_number)
    }

    pub fn validate_after(&self, previous: Option<ChainTip>) -> Result<(), BlockValidationError> {
        if self.point.hash == [0; 32] {
            return Err(BlockValidationError::EmptyHash);
        }
        if self.body_hash == [0; 32] {
            return Err(BlockValidationError::EmptyBodyHash);
        }
        if let Some(previous) = previous {
            if self.point.slot < previous.point.slot {
                return Err(BlockValidationError::SlotWentBackwards {
                    previous: previous.point.slot,
                    candidate: self.point.slot,
                });
            }
            if self.bundle_number <= previous.bundle_number {
                return Err(BlockValidationError::BundleNumberNotAdvanced {
                    previous: previous.bundle_number,
                    candidate: self.bundle_number,
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockItem {
    pub id: String,
    pub bytes: Vec<u8>,
}

impl BlockItem {
    pub fn new(id: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            bytes,
        }
    }

    pub fn validate(&self, max_bytes: usize) -> Result<(), BlockValidationError> {
        if self.id.trim().is_empty() {
            return Err(BlockValidationError::EmptyItemId);
        }
        if self.bytes.is_empty() {
            return Err(BlockValidationError::EmptyItemBytes);
        }
        if self.bytes.len() > max_bytes {
            return Err(BlockValidationError::ItemTooLarge {
                max_bytes,
                actual: self.bytes.len(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockBundle {
    pub header: BlockHeader,
    pub items: Vec<BlockItem>,
}

impl BlockBundle {
    pub fn validate(
        &self,
        previous: Option<ChainTip>,
        max_item_bytes: usize,
    ) -> Result<(), BlockValidationError> {
        self.header.validate_after(previous)?;
        if self.items.is_empty() {
            return Err(BlockValidationError::EmptyBundle);
        }
        for item in &self.items {
            item.validate(max_item_bytes)?;
        }
        Ok(())
    }

    pub fn validate_for_age(
        &self,
        previous: Option<ChainTip>,
        rule: BlockEraRule,
    ) -> Result<(), BlockValidationError> {
        rule.validate()?;
        if self.items.len() > rule.max_items {
            return Err(BlockValidationError::TooManyItems {
                max: rule.max_items,
                actual: self.items.len(),
            });
        }
        if rule.require_parent_hash && self.header.parent_hash == [0; 32] {
            return Err(BlockValidationError::EmptyParentHash);
        }
        self.validate(previous, rule.max_item_bytes)
    }

    pub fn validate_local_fixture_integrity(
        &self,
        previous: Option<ChainTip>,
    ) -> Result<(), BlockValidationError> {
        if let Some(previous) = previous {
            if self.header.parent_hash != previous.point.hash {
                return Err(BlockValidationError::ParentHashMismatch {
                    expected: previous.point.hash,
                    actual: self.header.parent_hash,
                });
            }
        }
        let expected = local_fixture_body_hash(&self.items);
        if self.header.body_hash != expected {
            return Err(BlockValidationError::BodyHashMismatch {
                expected,
                actual: self.header.body_hash,
            });
        }
        Ok(())
    }

    pub fn encode_local(&self) -> Result<Vec<u8>, BlockDecodeError> {
        let item_count =
            u32::try_from(self.items.len()).map_err(|_| BlockDecodeError::LengthTooLarge {
                field: "items",
                actual: self.items.len(),
            })?;
        let mut out = Vec::new();
        out.extend_from_slice(LOCAL_BLOCK_MAGIC);
        out.extend_from_slice(&self.header.point.slot.to_be_bytes());
        out.extend_from_slice(&self.header.point.hash);
        out.extend_from_slice(&self.header.bundle_number.to_be_bytes());
        out.extend_from_slice(&self.header.parent_hash);
        out.extend_from_slice(&self.header.body_hash);
        out.extend_from_slice(&item_count.to_be_bytes());
        for item in &self.items {
            let id = item.id.as_bytes();
            let id_len = u32::try_from(id.len()).map_err(|_| BlockDecodeError::LengthTooLarge {
                field: "item_id",
                actual: id.len(),
            })?;
            let bytes_len =
                u32::try_from(item.bytes.len()).map_err(|_| BlockDecodeError::LengthTooLarge {
                    field: "item_bytes",
                    actual: item.bytes.len(),
                })?;
            out.extend_from_slice(&id_len.to_be_bytes());
            out.extend_from_slice(id);
            out.extend_from_slice(&bytes_len.to_be_bytes());
            out.extend_from_slice(&item.bytes);
        }
        Ok(out)
    }

    pub fn decode_local(data: &[u8]) -> Result<Self, BlockDecodeError> {
        let mut cursor = DecodeCursor::new(data);
        let magic = cursor.read_exact(LOCAL_BLOCK_MAGIC.len())?;
        if magic != LOCAL_BLOCK_MAGIC {
            return Err(BlockDecodeError::InvalidMagic);
        }
        let slot = cursor.read_u64()?;
        let point_hash = cursor.read_hash()?;
        let bundle_number = cursor.read_u64()?;
        let parent_hash = cursor.read_hash()?;
        let body_hash = cursor.read_hash()?;
        let item_count = cursor.read_u32()? as usize;
        if item_count > MAX_DECODED_ITEMS {
            return Err(BlockDecodeError::TooManyEncodedItems {
                max: MAX_DECODED_ITEMS,
                actual: item_count,
            });
        }
        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            let id_len = cursor.read_u32()? as usize;
            let id_bytes = cursor.read_exact(id_len)?;
            let id =
                String::from_utf8(id_bytes.to_vec()).map_err(|_| BlockDecodeError::InvalidUtf8)?;
            let bytes_len = cursor.read_u32()? as usize;
            let bytes = cursor.read_exact(bytes_len)?.to_vec();
            items.push(BlockItem::new(id, bytes));
        }
        if cursor.remaining() != 0 {
            return Err(BlockDecodeError::TrailingBytes(cursor.remaining()));
        }
        Ok(Self {
            header: BlockHeader {
                point: ChainPoint::new(slot, point_hash),
                bundle_number,
                parent_hash,
                body_hash,
            },
            items,
        })
    }

    pub fn decode_local_and_validate(
        data: &[u8],
        previous: Option<ChainTip>,
        rule: BlockEraRule,
    ) -> Result<Self, BlockDecodeError> {
        let bundle = Self::decode_local(data)?;
        bundle.validate_for_age(previous, rule)?;
        Ok(bundle)
    }

    pub fn decode_local_fixture_and_validate(
        data: &[u8],
        previous: Option<ChainTip>,
        rule: BlockEraRule,
    ) -> Result<Self, BlockDecodeError> {
        let bundle = Self::decode_local(data)?;
        bundle.validate_for_age(previous, rule)?;
        bundle.validate_local_fixture_integrity(previous)?;
        Ok(bundle)
    }
}

pub fn local_fixture_body_hash(items: &[BlockItem]) -> [u8; 32] {
    let mut state = [
        0xcbf2_9ce4_8422_2325_u64,
        0x9e37_79b9_7f4a_7c15_u64,
        0x94d0_49bb_1331_11eb_u64,
        0x2545_f491_4f6c_dd1d_u64,
    ];
    mix_local_fixture_u64(&mut state, items.len() as u64);
    for item in items {
        mix_local_fixture_u64(&mut state, item.id.len() as u64);
        mix_local_fixture_bytes(&mut state, item.id.as_bytes());
        mix_local_fixture_u64(&mut state, item.bytes.len() as u64);
        mix_local_fixture_bytes(&mut state, &item.bytes);
    }
    let mut out = [0; 32];
    for (index, word) in state.into_iter().enumerate() {
        out[index * 8..index * 8 + 8].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn mix_local_fixture_u64(state: &mut [u64; 4], value: u64) {
    for byte in value.to_be_bytes() {
        mix_local_fixture_byte(state, byte);
    }
}

fn mix_local_fixture_bytes(state: &mut [u64; 4], bytes: &[u8]) {
    for byte in bytes {
        mix_local_fixture_byte(state, *byte);
    }
}

fn mix_local_fixture_byte(state: &mut [u64; 4], byte: u8) {
    for (index, word) in state.iter_mut().enumerate() {
        let salt = ((index as u64) << 56) | byte as u64;
        *word ^= salt;
        *word = word.wrapping_mul(0x0000_0100_0000_01b3 + index as u64 * 0x100);
        *word ^= *word >> 32;
    }
}

struct DecodeCursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> DecodeCursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], BlockDecodeError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(BlockDecodeError::UnexpectedEof {
                needed: len,
                remaining: self.remaining(),
            })?;
        if end > self.data.len() {
            return Err(BlockDecodeError::UnexpectedEof {
                needed: len,
                remaining: self.remaining(),
            });
        }
        let slice = &self.data[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn read_u32(&mut self) -> Result<u32, BlockDecodeError> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_be_bytes(
            bytes.try_into().expect("slice length checked"),
        ))
    }

    fn read_u64(&mut self) -> Result<u64, BlockDecodeError> {
        let bytes = self.read_exact(8)?;
        Ok(u64::from_be_bytes(
            bytes.try_into().expect("slice length checked"),
        ))
    }

    fn read_hash(&mut self) -> Result<[u8; 32], BlockDecodeError> {
        let bytes = self.read_exact(32)?;
        let mut hash = [0; 32];
        hash.copy_from_slice(bytes);
        Ok(hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDecodeError {
    InvalidMagic,
    UnexpectedEof { needed: usize, remaining: usize },
    InvalidUtf8,
    TrailingBytes(usize),
    TooManyEncodedItems { max: usize, actual: usize },
    LengthTooLarge { field: &'static str, actual: usize },
    Validation(BlockValidationError),
}

impl fmt::Display for BlockDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => f.write_str("invalid local block magic"),
            Self::UnexpectedEof { needed, remaining } => write!(
                f,
                "unexpected end of local block: needed {needed} bytes, remaining {remaining}"
            ),
            Self::InvalidUtf8 => f.write_str("local block item id is not valid UTF-8"),
            Self::TrailingBytes(bytes) => write!(f, "local block has {bytes} trailing bytes"),
            Self::TooManyEncodedItems { max, actual } => {
                write!(
                    f,
                    "local block item count too large: max={max} actual={actual}"
                )
            }
            Self::LengthTooLarge { field, actual } => {
                write!(f, "local block {field} length too large: {actual}")
            }
            Self::Validation(err) => write!(f, "decoded block validation failed: {err}"),
        }
    }
}

impl BlockDecodeError {
    pub fn summary_line(&self) -> String {
        match self {
            Self::InvalidMagic => "block_decode_failed reason=invalid_magic".to_string(),
            Self::UnexpectedEof { needed, remaining } => format!(
                "block_decode_failed reason=unexpected_eof needed={needed} remaining={remaining}"
            ),
            Self::InvalidUtf8 => "block_decode_failed reason=invalid_utf8".to_string(),
            Self::TrailingBytes(bytes) => {
                format!("block_decode_failed reason=trailing_bytes bytes={bytes}")
            }
            Self::TooManyEncodedItems { max, actual } => format!(
                "block_decode_failed reason=too_many_encoded_items max={max} actual={actual}"
            ),
            Self::LengthTooLarge { field, actual } => {
                format!("block_decode_failed reason=length_too_large field={field} actual={actual}")
            }
            Self::Validation(_) => "block_decode_failed reason=validation".to_string(),
        }
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            BLOCK_DECODE_FAILED_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl std::error::Error for BlockDecodeError {}

impl From<BlockValidationError> for BlockDecodeError {
    fn from(value: BlockValidationError) -> Self {
        Self::Validation(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEraRule {
    pub age_id: u64,
    pub max_item_bytes: usize,
    pub max_items: usize,
    pub require_parent_hash: bool,
}

impl BlockEraRule {
    pub fn validate(self) -> Result<(), BlockValidationError> {
        if self.max_item_bytes == 0 {
            return Err(BlockValidationError::InvalidEraRule(
                "max_item_bytes must be > 0",
            ));
        }
        if self.max_items == 0 {
            return Err(BlockValidationError::InvalidEraRule(
                "max_items must be > 0",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEraFixture {
    pub name: &'static str,
    pub min_major_version: u64,
    pub max_major_version: u64,
    pub rule: BlockEraRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEraCatalog {
    pub eras: Vec<BlockEraFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEraValidationReport {
    pub major_version: u64,
    pub era_name: &'static str,
    pub age_id: u64,
    pub item_count: usize,
    pub max_items: usize,
    pub max_item_bytes: usize,
    pub parent_required: bool,
    pub fixture_integrity_checked: bool,
}

impl BlockEraValidationReport {
    pub fn summary_line(&self) -> String {
        format!(
            "block_era_validation major_version={} era={} age={} items={} max_items={} max_item_bytes={} parent_required={} fixture_integrity={}",
            self.major_version,
            self.era_name,
            self.age_id,
            self.item_count,
            self.max_items,
            self.max_item_bytes,
            self.parent_required,
            self.fixture_integrity_checked,
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            BLOCK_ERA_VALIDATION_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl BlockEraCatalog {
    pub fn new(eras: Vec<BlockEraFixture>) -> Self {
        Self { eras }
    }

    pub fn validate(&self) -> Result<(), BlockValidationError> {
        if self.eras.is_empty() {
            return Err(BlockValidationError::EmptyEraCatalog);
        }
        for (index, era) in self.eras.iter().enumerate() {
            if era.name.trim().is_empty() {
                return Err(BlockValidationError::InvalidEraCatalog(
                    "era name must not be empty".to_string(),
                ));
            }
            if era.min_major_version > era.max_major_version {
                return Err(BlockValidationError::InvalidEraCatalog(format!(
                    "era {} has min major version greater than max",
                    era.name
                )));
            }
            era.rule.validate()?;
            if index > 0 {
                let previous = &self.eras[index - 1];
                if era.min_major_version <= previous.max_major_version {
                    return Err(BlockValidationError::InvalidEraCatalog(format!(
                        "era {} overlaps or is out of order after {}",
                        era.name, previous.name
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn era_for_major_version(
        &self,
        major_version: u64,
    ) -> Result<&BlockEraFixture, BlockValidationError> {
        self.validate()?;
        self.eras
            .iter()
            .find(|era| {
                major_version >= era.min_major_version && major_version <= era.max_major_version
            })
            .ok_or(BlockValidationError::UnknownMajorVersion(major_version))
    }

    pub fn rule_for_major_version(
        &self,
        major_version: u64,
    ) -> Result<BlockEraRule, BlockValidationError> {
        self.era_for_major_version(major_version)
            .map(|era| era.rule)
    }

    pub fn validate_bundle_for_major_version(
        &self,
        bundle: &BlockBundle,
        previous: Option<ChainTip>,
        major_version: u64,
    ) -> Result<(), BlockValidationError> {
        let rule = self.rule_for_major_version(major_version)?;
        bundle.validate_for_age(previous, rule)
    }

    pub fn validate_bundle_report_for_major_version(
        &self,
        bundle: &BlockBundle,
        previous: Option<ChainTip>,
        major_version: u64,
    ) -> Result<BlockEraValidationReport, BlockValidationError> {
        let era = self.era_for_major_version(major_version)?;
        bundle.validate_for_age(previous, era.rule)?;
        Ok(BlockEraValidationReport {
            major_version,
            era_name: era.name,
            age_id: era.rule.age_id,
            item_count: bundle.items.len(),
            max_items: era.rule.max_items,
            max_item_bytes: era.rule.max_item_bytes,
            parent_required: era.rule.require_parent_hash,
            fixture_integrity_checked: false,
        })
    }

    pub fn decode_local_bundle_report_for_major_version(
        &self,
        data: &[u8],
        previous: Option<ChainTip>,
        major_version: u64,
    ) -> Result<(BlockBundle, BlockEraValidationReport), BlockDecodeError> {
        let bundle = BlockBundle::decode_local(data)?;
        let report =
            self.validate_bundle_report_for_major_version(&bundle, previous, major_version)?;
        Ok((bundle, report))
    }

    pub fn validate_local_fixture_bundle_for_major_version(
        &self,
        bundle: &BlockBundle,
        previous: Option<ChainTip>,
        major_version: u64,
    ) -> Result<(), BlockValidationError> {
        self.validate_bundle_for_major_version(bundle, previous, major_version)?;
        bundle.validate_local_fixture_integrity(previous)
    }

    pub fn validate_local_fixture_bundle_report_for_major_version(
        &self,
        bundle: &BlockBundle,
        previous: Option<ChainTip>,
        major_version: u64,
    ) -> Result<BlockEraValidationReport, BlockValidationError> {
        let era = self.era_for_major_version(major_version)?;
        bundle.validate_for_age(previous, era.rule)?;
        bundle.validate_local_fixture_integrity(previous)?;
        Ok(BlockEraValidationReport {
            major_version,
            era_name: era.name,
            age_id: era.rule.age_id,
            item_count: bundle.items.len(),
            max_items: era.rule.max_items,
            max_item_bytes: era.rule.max_item_bytes,
            parent_required: era.rule.require_parent_hash,
            fixture_integrity_checked: true,
        })
    }

    pub fn decode_local_fixture_bundle_report_for_major_version(
        &self,
        data: &[u8],
        previous: Option<ChainTip>,
        major_version: u64,
    ) -> Result<(BlockBundle, BlockEraValidationReport), BlockDecodeError> {
        let bundle = BlockBundle::decode_local(data)?;
        let report = self.validate_local_fixture_bundle_report_for_major_version(
            &bundle,
            previous,
            major_version,
        )?;
        Ok((bundle, report))
    }
}

pub fn local_ledger_fixture_catalog() -> BlockEraCatalog {
    BlockEraCatalog::new(vec![
        BlockEraFixture {
            name: "fixture-bootstrap",
            min_major_version: 0,
            max_major_version: 0,
            rule: BlockEraRule {
                age_id: 0,
                max_item_bytes: 16 * 1024,
                max_items: 1,
                require_parent_hash: false,
            },
        },
        BlockEraFixture {
            name: "fixture-transition",
            min_major_version: 1,
            max_major_version: 5,
            rule: BlockEraRule {
                age_id: 1,
                max_item_bytes: 64 * 1024,
                max_items: 1_024,
                require_parent_hash: true,
            },
        },
        BlockEraFixture {
            name: "fixture-current",
            min_major_version: 6,
            max_major_version: 9,
            rule: BlockEraRule {
                age_id: 2,
                max_item_bytes: 128 * 1024,
                max_items: 4_096,
                require_parent_hash: true,
            },
        },
    ])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockValidationError {
    EmptyHash,
    EmptyParentHash,
    EmptyBodyHash,
    SlotWentBackwards {
        previous: u64,
        candidate: u64,
    },
    BundleNumberNotAdvanced {
        previous: u64,
        candidate: u64,
    },
    EmptyItemId,
    EmptyItemBytes,
    ItemTooLarge {
        max_bytes: usize,
        actual: usize,
    },
    TooManyItems {
        max: usize,
        actual: usize,
    },
    EmptyBundle,
    ParentHashMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    BodyHashMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    InvalidEraRule(&'static str),
    EmptyEraCatalog,
    InvalidEraCatalog(String),
    UnknownMajorVersion(u64),
}

impl fmt::Display for BlockValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyHash => f.write_str("block header hash must not be empty"),
            Self::EmptyParentHash => f.write_str("block header parent hash must not be empty"),
            Self::EmptyBodyHash => f.write_str("block header body hash must not be empty"),
            Self::SlotWentBackwards {
                previous,
                candidate,
            } => {
                write!(
                    f,
                    "block slot went backwards: previous={previous} candidate={candidate}"
                )
            }
            Self::BundleNumberNotAdvanced {
                previous,
                candidate,
            } => write!(
                f,
                "block number did not advance: previous={previous} candidate={candidate}"
            ),
            Self::EmptyItemId => f.write_str("block item id must not be empty"),
            Self::EmptyItemBytes => f.write_str("block item bytes must not be empty"),
            Self::ItemTooLarge { max_bytes, actual } => {
                write!(f, "block item too large: max={max_bytes} actual={actual}")
            }
            Self::TooManyItems { max, actual } => {
                write!(f, "block has too many items: max={max} actual={actual}")
            }
            Self::EmptyBundle => f.write_str("block must contain at least one item"),
            Self::ParentHashMismatch { expected, actual } => write!(
                f,
                "block parent hash mismatch: expected={} actual={}",
                short_hash(expected),
                short_hash(actual)
            ),
            Self::BodyHashMismatch { expected, actual } => write!(
                f,
                "block body hash mismatch: expected={} actual={}",
                short_hash(expected),
                short_hash(actual)
            ),
            Self::InvalidEraRule(reason) => write!(f, "invalid block era rule: {reason}"),
            Self::EmptyEraCatalog => f.write_str("block era catalog must not be empty"),
            Self::InvalidEraCatalog(reason) => write!(f, "invalid block era catalog: {reason}"),
            Self::UnknownMajorVersion(version) => {
                write!(f, "no block era rule for major version {version}")
            }
        }
    }
}

impl BlockValidationError {
    pub fn summary_line(&self) -> String {
        match self {
            Self::EmptyHash => "block_era_validation_failed reason=empty_hash".to_string(),
            Self::EmptyParentHash => {
                "block_era_validation_failed reason=empty_parent_hash".to_string()
            }
            Self::EmptyBodyHash => {
                "block_era_validation_failed reason=empty_body_hash".to_string()
            }
            Self::SlotWentBackwards {
                previous,
                candidate,
            } => format!(
                "block_era_validation_failed reason=slot_went_backwards previous={previous} candidate={candidate}"
            ),
            Self::BundleNumberNotAdvanced {
                previous,
                candidate,
            } => format!(
                "block_era_validation_failed reason=bundle_number_not_advanced previous={previous} candidate={candidate}"
            ),
            Self::EmptyItemId => "block_era_validation_failed reason=empty_item_id".to_string(),
            Self::EmptyItemBytes => {
                "block_era_validation_failed reason=empty_item_bytes".to_string()
            }
            Self::ItemTooLarge { max_bytes, actual } => format!(
                "block_era_validation_failed reason=item_too_large max_bytes={max_bytes} actual={actual}"
            ),
            Self::TooManyItems { max, actual } => format!(
                "block_era_validation_failed reason=too_many_items max={max} actual={actual}"
            ),
            Self::EmptyBundle => "block_era_validation_failed reason=empty_bundle".to_string(),
            Self::ParentHashMismatch { expected, actual } => format!(
                "block_era_validation_failed reason=parent_hash_mismatch expected={} actual={}",
                short_hash(expected),
                short_hash(actual)
            ),
            Self::BodyHashMismatch { expected, actual } => format!(
                "block_era_validation_failed reason=body_hash_mismatch expected={} actual={}",
                short_hash(expected),
                short_hash(actual)
            ),
            Self::InvalidEraRule(_) => {
                "block_era_validation_failed reason=invalid_era_rule".to_string()
            }
            Self::EmptyEraCatalog => {
                "block_era_validation_failed reason=empty_era_catalog".to_string()
            }
            Self::InvalidEraCatalog(_) => {
                "block_era_validation_failed reason=invalid_era_catalog".to_string()
            }
            Self::UnknownMajorVersion(version) => format!(
                "block_era_validation_failed reason=unknown_major_version major_version={version}"
            ),
        }
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            BLOCK_ERA_VALIDATION_FAILED_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl std::error::Error for BlockValidationError {}

fn short_hash(hash: &[u8; 32]) -> String {
    let mut out = String::with_capacity(8);
    for byte in &hash[..4] {
        out.push(hex_nibble(byte >> 4));
        out.push(hex_nibble(byte & 0x0f));
    }
    out
}

fn hex_nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => '?',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(slot: u64, bundle_number: u64) -> BlockHeader {
        BlockHeader {
            point: ChainPoint::new(slot, [1; 32]),
            bundle_number,
            parent_hash: [2; 32],
            body_hash: [3; 32],
        }
    }

    fn fixture_bundle(previous: ChainTip) -> BlockBundle {
        let items = vec![
            BlockItem::new("tx-a", b"local-tx-a".to_vec()),
            BlockItem::new("tx-b", b"local-tx-b".to_vec()),
        ];
        BlockBundle {
            header: BlockHeader {
                point: ChainPoint::new(previous.point.slot + 1, [7; 32]),
                bundle_number: previous.bundle_number + 1,
                parent_hash: previous.point.hash,
                body_hash: local_fixture_body_hash(&items),
            },
            items,
        }
    }

    #[test]
    fn block_bundle_validates_local_shape() {
        let bundle = BlockBundle {
            header: header(2, 2),
            items: vec![BlockItem::new("item", vec![1])],
        };
        assert_eq!(
            bundle.validate(Some(ChainTip::new(ChainPoint::new(1, [9; 32]), 1)), 10),
            Ok(())
        );
    }

    #[test]
    fn block_header_rejects_backward_progress() {
        let bundle = BlockBundle {
            header: header(1, 1),
            items: vec![BlockItem::new("item", vec![1])],
        };
        assert_eq!(
            bundle.validate(Some(ChainTip::new(ChainPoint::new(2, [9; 32]), 2)), 10),
            Err(BlockValidationError::SlotWentBackwards {
                previous: 2,
                candidate: 1,
            })
        );
    }

    #[test]
    fn block_era_rule_caps_items_and_requires_parent_hash_locally() {
        let mut bundle = BlockBundle {
            header: header(2, 2),
            items: vec![BlockItem::new("a", vec![1]), BlockItem::new("b", vec![2])],
        };
        let rule = BlockEraRule {
            age_id: 2,
            max_item_bytes: 10,
            max_items: 1,
            require_parent_hash: true,
        };
        assert_eq!(
            bundle.validate_for_age(None, rule),
            Err(BlockValidationError::TooManyItems { max: 1, actual: 2 })
        );
        bundle.items.truncate(1);
        bundle.header.parent_hash = [0; 32];
        assert_eq!(
            bundle.validate_for_age(None, rule),
            Err(BlockValidationError::EmptyParentHash)
        );
    }

    #[test]
    fn local_fixture_catalog_selects_rules_by_major_version() {
        let catalog = local_ledger_fixture_catalog();
        assert_eq!(catalog.validate(), Ok(()));
        assert_eq!(
            catalog.era_for_major_version(5).unwrap().name,
            "fixture-transition"
        );
        assert_eq!(catalog.rule_for_major_version(0).unwrap().age_id, 0);
        assert_eq!(catalog.rule_for_major_version(5).unwrap().age_id, 1);
        assert_eq!(catalog.rule_for_major_version(9).unwrap().age_id, 2);
        assert_eq!(
            catalog.rule_for_major_version(10),
            Err(BlockValidationError::UnknownMajorVersion(10))
        );
    }

    #[test]
    fn local_fixture_catalog_validates_bundle_with_selected_rule() {
        let catalog = local_ledger_fixture_catalog();
        let mut bundle = BlockBundle {
            header: header(2, 2),
            items: vec![BlockItem::new("tx", vec![1, 2, 3])],
        };

        assert_eq!(
            catalog.validate_bundle_for_major_version(&bundle, None, 6),
            Ok(())
        );
        bundle.header.parent_hash = [0; 32];
        assert_eq!(
            catalog.validate_bundle_for_major_version(&bundle, None, 6),
            Err(BlockValidationError::EmptyParentHash)
        );
        assert_eq!(
            catalog.validate_bundle_for_major_version(&bundle, None, 0),
            Ok(())
        );
    }

    #[test]
    fn local_fixture_integrity_binds_parent_and_body_hashes() {
        let previous = ChainTip::new(ChainPoint::new(9, [4; 32]), 11);
        let bundle = fixture_bundle(previous);

        assert_eq!(
            bundle.validate_local_fixture_integrity(Some(previous)),
            Ok(())
        );
    }

    #[test]
    fn local_fixture_integrity_rejects_parent_mismatch() {
        let previous = ChainTip::new(ChainPoint::new(9, [4; 32]), 11);
        let mut bundle = fixture_bundle(previous);
        bundle.header.parent_hash = [8; 32];

        assert_eq!(
            bundle.validate_local_fixture_integrity(Some(previous)),
            Err(BlockValidationError::ParentHashMismatch {
                expected: [4; 32],
                actual: [8; 32],
            })
        );
    }

    #[test]
    fn local_fixture_integrity_rejects_body_hash_mismatch() {
        let previous = ChainTip::new(ChainPoint::new(9, [4; 32]), 11);
        let mut bundle = fixture_bundle(previous);
        let expected = bundle.header.body_hash;
        bundle.header.body_hash = [8; 32];

        assert_eq!(
            bundle.validate_local_fixture_integrity(Some(previous)),
            Err(BlockValidationError::BodyHashMismatch {
                expected,
                actual: [8; 32],
            })
        );
    }

    #[test]
    fn local_fixture_catalog_can_validate_fixture_integrity() {
        let catalog = local_ledger_fixture_catalog();
        let previous = ChainTip::new(ChainPoint::new(9, [4; 32]), 11);
        let bundle = fixture_bundle(previous);

        assert_eq!(
            catalog.validate_local_fixture_bundle_for_major_version(&bundle, Some(previous), 6),
            Ok(())
        );
    }

    #[test]
    fn local_fixture_catalog_reports_validated_bundle_shape() {
        let catalog = local_ledger_fixture_catalog();
        let previous = ChainTip::new(ChainPoint::new(9, [4; 32]), 11);
        let bundle = fixture_bundle(previous);

        let report = catalog
            .validate_local_fixture_bundle_report_for_major_version(&bundle, Some(previous), 6)
            .unwrap();

        assert_eq!(report.major_version, 6);
        assert_eq!(report.era_name, "fixture-current");
        assert_eq!(report.age_id, 2);
        assert_eq!(report.item_count, 2);
        assert_eq!(report.max_items, 4_096);
        assert_eq!(report.max_item_bytes, 128 * 1024);
        assert!(report.parent_required);
        assert!(report.fixture_integrity_checked);
        assert_eq!(
            report.summary_line(),
            "block_era_validation major_version=6 era=fixture-current age=2 items=2 max_items=4096 max_item_bytes=131072 parent_required=true fixture_integrity=true"
        );
        let event = report.to_event();
        let events = report.events();
        assert_eq!(event.name.as_str(), BLOCK_ERA_VALIDATION_EVENT);
        assert_eq!(event.payload, EventPayload::Text(report.summary_line()));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), BLOCK_ERA_VALIDATION_EVENT);
        assert_eq!(events[0].payload, EventPayload::Text(report.summary_line()));

        let rule_only_report = catalog
            .validate_bundle_report_for_major_version(&bundle, Some(previous), 6)
            .unwrap();
        assert!(!rule_only_report.fixture_integrity_checked);
    }

    #[test]
    fn block_validation_error_emits_local_failure_event() {
        let err = BlockValidationError::TooManyItems { max: 1, actual: 2 };
        let event = err.to_event();
        let events = err.events();

        assert_eq!(
            err.summary_line(),
            "block_era_validation_failed reason=too_many_items max=1 actual=2"
        );
        assert_eq!(event.name.as_str(), BLOCK_ERA_VALIDATION_FAILED_EVENT);
        assert_eq!(event.payload, EventPayload::Text(err.summary_line()));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), BLOCK_ERA_VALIDATION_FAILED_EVENT);
        assert_eq!(events[0].payload, EventPayload::Text(err.summary_line()));
    }

    #[test]
    fn local_block_codec_can_validate_decoded_fixture_integrity() {
        let previous = ChainTip::new(ChainPoint::new(9, [4; 32]), 11);
        let bundle = fixture_bundle(previous);
        let rule = local_ledger_fixture_catalog()
            .rule_for_major_version(6)
            .unwrap();

        assert_eq!(
            BlockBundle::decode_local_fixture_and_validate(
                &bundle.encode_local().unwrap(),
                Some(previous),
                rule
            )
            .unwrap(),
            bundle
        );
    }

    #[test]
    fn local_block_codec_can_decode_and_report_fixture_validation() {
        let catalog = local_ledger_fixture_catalog();
        let previous = ChainTip::new(ChainPoint::new(9, [4; 32]), 11);
        let bundle = fixture_bundle(previous);
        let encoded = bundle.encode_local().unwrap();

        let (decoded, report) = catalog
            .decode_local_fixture_bundle_report_for_major_version(&encoded, Some(previous), 6)
            .unwrap();

        assert_eq!(decoded, bundle);
        assert_eq!(report.major_version, 6);
        assert_eq!(report.era_name, "fixture-current");
        assert!(report.fixture_integrity_checked);

        let (_, rule_only_report) = catalog
            .decode_local_bundle_report_for_major_version(&encoded, Some(previous), 6)
            .unwrap();
        assert!(!rule_only_report.fixture_integrity_checked);
    }

    #[test]
    fn block_era_catalog_rejects_overlapping_fixture_ranges() {
        let catalog = BlockEraCatalog::new(vec![
            BlockEraFixture {
                name: "first",
                min_major_version: 0,
                max_major_version: 2,
                rule: BlockEraRule {
                    age_id: 0,
                    max_item_bytes: 1,
                    max_items: 1,
                    require_parent_hash: false,
                },
            },
            BlockEraFixture {
                name: "second",
                min_major_version: 2,
                max_major_version: 3,
                rule: BlockEraRule {
                    age_id: 1,
                    max_item_bytes: 1,
                    max_items: 1,
                    require_parent_hash: false,
                },
            },
        ]);

        assert!(matches!(
            catalog.validate(),
            Err(BlockValidationError::InvalidEraCatalog(_))
        ));
    }

    #[test]
    fn local_block_codec_round_trips_header_and_items() {
        let bundle = BlockBundle {
            header: header(7, 4),
            items: vec![
                BlockItem::new("tx-1", vec![1, 2, 3]),
                BlockItem::new("tx-2", vec![4, 5]),
            ],
        };
        let encoded = bundle.encode_local().unwrap();
        assert!(encoded.starts_with(LOCAL_BLOCK_MAGIC));
        assert_eq!(BlockBundle::decode_local(&encoded).unwrap(), bundle);
    }

    #[test]
    fn local_block_codec_rejects_truncated_input() {
        let bundle = BlockBundle {
            header: header(7, 4),
            items: vec![BlockItem::new("tx", vec![1, 2, 3])],
        };
        let mut encoded = bundle.encode_local().unwrap();
        encoded.truncate(encoded.len() - 2);
        assert!(matches!(
            BlockBundle::decode_local(&encoded),
            Err(BlockDecodeError::UnexpectedEof { .. })
        ));
    }

    #[test]
    fn block_decode_error_emits_local_failure_event() {
        let err = BlockDecodeError::UnexpectedEof {
            needed: 4,
            remaining: 1,
        };
        let event = err.to_event();
        let events = err.events();

        assert_eq!(
            err.summary_line(),
            "block_decode_failed reason=unexpected_eof needed=4 remaining=1"
        );
        assert_eq!(event.name.as_str(), BLOCK_DECODE_FAILED_EVENT);
        assert_eq!(event.payload, EventPayload::Text(err.summary_line()));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), BLOCK_DECODE_FAILED_EVENT);
        assert_eq!(events[0].payload, EventPayload::Text(err.summary_line()));
    }

    #[test]
    fn local_block_codec_rejects_trailing_input() {
        let bundle = BlockBundle {
            header: header(7, 4),
            items: vec![BlockItem::new("tx", vec![1, 2, 3])],
        };
        let mut encoded = bundle.encode_local().unwrap();
        encoded.push(0xff);
        assert_eq!(
            BlockBundle::decode_local(&encoded),
            Err(BlockDecodeError::TrailingBytes(1))
        );
    }

    #[test]
    fn local_block_codec_can_validate_decoded_bundle() {
        let bundle = BlockBundle {
            header: header(2, 2),
            items: vec![BlockItem::new("tx", vec![1, 2, 3])],
        };
        let rule = BlockEraRule {
            age_id: 0,
            max_item_bytes: 2,
            max_items: 10,
            require_parent_hash: true,
        };
        assert_eq!(
            BlockBundle::decode_local_and_validate(&bundle.encode_local().unwrap(), None, rule),
            Err(BlockDecodeError::Validation(
                BlockValidationError::ItemTooLarge {
                    max_bytes: 2,
                    actual: 3,
                }
            ))
        );
    }
}
