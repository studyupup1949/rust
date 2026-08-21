use std::fmt;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossingTriggerKind {
    AtVersion,
    AtEpoch,
    NotInThisRite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrossingTrigger {
    pub kind: CrossingTriggerKind,
    pub version: u64,
    pub epoch: u64,
}

impl CrossingTrigger {
    pub fn at_version(major_version: u64) -> Self {
        Self {
            kind: CrossingTriggerKind::AtVersion,
            version: major_version,
            epoch: 0,
        }
    }

    pub fn at_epoch(epoch: u64) -> Self {
        Self {
            kind: CrossingTriggerKind::AtEpoch,
            version: 0,
            epoch,
        }
    }

    pub fn not_in_this_rite() -> Self {
        Self {
            kind: CrossingTriggerKind::NotInThisRite,
            version: 0,
            epoch: 0,
        }
    }
}

impl fmt::Display for CrossingTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            CrossingTriggerKind::AtVersion => write!(f, "AtVersion({})", self.version),
            CrossingTriggerKind::AtEpoch => write!(f, "AtEpoch({})", self.epoch),
            CrossingTriggerKind::NotInThisRite => f.write_str("NotInThisRite"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgeParams {
    pub epoch_size: u64,
    pub slot_length: Duration,
    pub safe_zone_slots: u64,
    pub first_light_window: u64,
}

impl AgeParams {
    pub fn validate(self) -> Result<(), CrossingError> {
        if self.epoch_size == 0 {
            return Err(CrossingError::InvalidAgeParams(
                "epoch_size must be > 0".to_string(),
            ));
        }
        if self.slot_length.is_zero() {
            return Err(CrossingError::InvalidAgeParams(
                "slot_length must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OghamBound {
    pub relative_time: Duration,
    pub slot: u64,
    pub epoch: u64,
}

impl OghamBound {
    pub const ZERO: Self = Self {
        relative_time: Duration::ZERO,
        slot: 0,
        epoch: 0,
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OghamEntry {
    pub age_id: u64,
    pub age_name: String,
    pub min_major_version: u64,
    pub max_major_version: u64,
    pub params: AgeParams,
    pub next_trigger: CrossingTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OghamShape {
    pub system_start: SystemTime,
    pub ages: Vec<OghamEntry>,
}

impl OghamShape {
    pub fn validate(&self) -> Result<(), CrossingError> {
        if self.ages.is_empty() {
            return Err(CrossingError::EmptyShape);
        }
        for (index, age) in self.ages.iter().enumerate() {
            if age.min_major_version > age.max_major_version {
                return Err(CrossingError::InvalidShape(format!(
                    "age {} has min major version greater than max major version",
                    age.age_name
                )));
            }
            age.params.validate()?;
            let is_last = index == self.ages.len() - 1;
            match (is_last, age.next_trigger.kind) {
                (true, CrossingTriggerKind::NotInThisRite) => {}
                (true, other) => {
                    return Err(CrossingError::InvalidShape(format!(
                        "final age {} must use NotInThisRite, got {:?}",
                        age.age_name, other
                    )));
                }
                (false, CrossingTriggerKind::NotInThisRite) => {
                    return Err(CrossingError::InvalidShape(format!(
                        "non-final age {} must not use NotInThisRite",
                        age.age_name
                    )));
                }
                (false, CrossingTriggerKind::AtVersion | CrossingTriggerKind::AtEpoch) => {}
            }
            if index > 0 {
                let previous = &self.ages[index - 1];
                if age.min_major_version == 0
                    || age.min_major_version - 1 != previous.max_major_version
                {
                    return Err(CrossingError::InvalidShape(format!(
                        "age {} min major version must be previous max + 1",
                        age.age_name
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn age_for_version(&self, major_version: u64) -> Option<&OghamEntry> {
        self.ages.iter().find(|age| {
            major_version >= age.min_major_version && major_version <= age.max_major_version
        })
    }

    pub fn age_for_id(&self, age_id: u64) -> Option<&OghamEntry> {
        self.ages.iter().find(|age| age.age_id == age_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossingState {
    Unknown,
    Known,
    Impossible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrossingInfo {
    pub state: CrossingState,
    pub known_epoch: u64,
}

impl CrossingInfo {
    pub fn unknown() -> Self {
        Self {
            state: CrossingState::Unknown,
            known_epoch: 0,
        }
    }

    pub fn known(epoch: u64) -> Self {
        Self {
            state: CrossingState::Known,
            known_epoch: epoch,
        }
    }

    pub fn impossible() -> Self {
        Self {
            state: CrossingState::Impossible,
            known_epoch: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgeSummary {
    pub age_id: u64,
    pub start: OghamBound,
    pub end: Option<OghamBound>,
    pub params: AgeParams,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochInfo {
    pub epoch: u64,
    pub start_slot: u64,
    pub length_in_slots: u64,
    pub slot_length: Duration,
    pub age_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OghamSummary {
    pub system_start: SystemTime,
    pub ages: Vec<AgeSummary>,
    pub crossing: CrossingInfo,
}

impl OghamSummary {
    pub fn current_age(&self) -> Option<&AgeSummary> {
        self.ages.last()
    }

    pub fn validate(&self) -> Result<(), CrossingError> {
        if self.ages.is_empty() {
            return Err(CrossingError::EmptySummary);
        }
        for (index, age) in self.ages.iter().enumerate() {
            age.params.validate()?;
            if index > 0 {
                let previous = &self.ages[index - 1];
                let Some(previous_end) = previous.end else {
                    return Err(CrossingError::UnboundedAgeNotLast(previous.age_id));
                };
                if previous_end != age.start {
                    return Err(CrossingError::AgeBoundaryMismatch {
                        previous: previous.age_id,
                        current: age.age_id,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn slot_to_time(&self, slot: u64) -> Result<SystemTime, CrossingError> {
        let age = self.age_for_slot(slot)?;
        let slots_into_age = slot - age.start.slot;
        let in_age = duration_mul(age.params.slot_length, slots_into_age)?;
        let rel = age
            .start
            .relative_time
            .checked_add(in_age)
            .ok_or(CrossingError::DurationOverflow)?;
        self.system_start
            .checked_add(rel)
            .ok_or(CrossingError::DurationOverflow)
    }

    pub fn time_to_slot(&self, time: SystemTime) -> Result<u64, CrossingError> {
        let rel = time
            .duration_since(self.system_start)
            .map_err(|_| CrossingError::BeforeFirstMark)?;
        let age = self.age_for_relative_time(rel)?;
        let rel_in_age = rel
            .checked_sub(age.start.relative_time)
            .ok_or(CrossingError::BeforeFirstMark)?;
        let slots_in_age = rel_in_age.as_nanos() / age.params.slot_length.as_nanos();
        let slots_in_age = u64::try_from(slots_in_age).map_err(|_| CrossingError::SlotOverflow)?;
        age.start
            .slot
            .checked_add(slots_in_age)
            .ok_or(CrossingError::SlotOverflow)
    }

    pub fn slot_to_epoch(&self, slot: u64) -> Result<EpochInfo, CrossingError> {
        let age = self.age_for_slot(slot)?;
        let slots_into_age = slot - age.start.slot;
        let epochs_into_age = slots_into_age / age.params.epoch_size;
        Ok(EpochInfo {
            epoch: age.start.epoch + epochs_into_age,
            start_slot: age.start.slot + epochs_into_age * age.params.epoch_size,
            length_in_slots: age.params.epoch_size,
            slot_length: age.params.slot_length,
            age_id: age.age_id,
        })
    }

    fn age_for_slot(&self, slot: u64) -> Result<&AgeSummary, CrossingError> {
        for age in &self.ages {
            if slot < age.start.slot {
                return Err(CrossingError::PastHorizon);
            }
            if age.end.is_none_or(|end| slot < end.slot) {
                return Ok(age);
            }
        }
        Err(CrossingError::PastHorizon)
    }

    fn age_for_relative_time(&self, relative_time: Duration) -> Result<&AgeSummary, CrossingError> {
        for age in &self.ages {
            if relative_time < age.start.relative_time {
                return Err(CrossingError::PastHorizon);
            }
            if age.end.is_none_or(|end| relative_time < end.relative_time) {
                return Ok(age);
            }
        }
        Err(CrossingError::PastHorizon)
    }
}

pub fn build_ogham_summary(
    shape: &OghamShape,
    past: Vec<AgeSummary>,
    mut current: AgeSummary,
    tip_slot: u64,
    crossing: CrossingInfo,
) -> Result<OghamSummary, CrossingError> {
    current.params.validate()?;
    for past_age in &past {
        if past_age.end.is_none() {
            return Err(CrossingError::PastAgeUnbounded(past_age.age_id));
        }
    }

    current.end = match crossing.state {
        CrossingState::Known => {
            if crossing.known_epoch <= current.start.epoch {
                return Err(CrossingError::KnownCrossingNotAfterAgeStart {
                    known_epoch: crossing.known_epoch,
                    age_start_epoch: current.start.epoch,
                });
            }
            Some(mk_upper_bound(
                current.params,
                current.start,
                crossing.known_epoch,
            )?)
        }
        CrossingState::Unknown => {
            let from_slot = tip_slot.saturating_add(1).max(current.start.slot);
            apply_safe_zone(current.params, current.start, from_slot)?
        }
        CrossingState::Impossible => {
            apply_safe_zone(current.params, current.start, current.start.slot)?
        }
    };

    let mut ages = Vec::with_capacity(past.len() + 1);
    ages.extend(past);
    ages.push(current);
    Ok(OghamSummary {
        system_start: shape.system_start,
        ages,
        crossing,
    })
}

pub fn mk_upper_bound(
    params: AgeParams,
    lower_bound: OghamBound,
    high_epoch: u64,
) -> Result<OghamBound, CrossingError> {
    let epochs_in_age = high_epoch
        .checked_sub(lower_bound.epoch)
        .ok_or(CrossingError::SlotOverflow)?;
    let slots_in_age = epochs_in_age
        .checked_mul(params.epoch_size)
        .ok_or(CrossingError::SlotOverflow)?;
    let in_age_time = duration_mul(params.slot_length, slots_in_age)?;
    Ok(OghamBound {
        relative_time: lower_bound
            .relative_time
            .checked_add(in_age_time)
            .ok_or(CrossingError::DurationOverflow)?,
        slot: lower_bound
            .slot
            .checked_add(slots_in_age)
            .ok_or(CrossingError::SlotOverflow)?,
        epoch: high_epoch,
    })
}

pub fn slot_to_epoch_bound(params: AgeParams, lower_bound: OghamBound, high_slot: u64) -> u64 {
    if high_slot < lower_bound.slot {
        return lower_bound.epoch;
    }
    let slots_from_lower = high_slot - lower_bound.slot;
    let epochs = slots_from_lower / params.epoch_size;
    let bump = u64::from(!slots_from_lower.is_multiple_of(params.epoch_size));
    lower_bound.epoch + epochs + bump
}

pub fn apply_safe_zone(
    params: AgeParams,
    lower_bound: OghamBound,
    from_slot: u64,
) -> Result<Option<OghamBound>, CrossingError> {
    if params.safe_zone_slots == 0 {
        return Ok(None);
    }
    let target = from_slot
        .checked_add(params.safe_zone_slots)
        .ok_or(CrossingError::SlotOverflow)?;
    let high_epoch = slot_to_epoch_bound(params, lower_bound, target);
    Ok(Some(mk_upper_bound(params, lower_bound, high_epoch)?))
}

fn duration_mul(duration: Duration, by: u64) -> Result<Duration, CrossingError> {
    let nanos = duration
        .as_nanos()
        .checked_mul(by as u128)
        .ok_or(CrossingError::DurationOverflow)?;
    let secs = nanos / 1_000_000_000;
    let subsec_nanos = nanos % 1_000_000_000;
    let secs = u64::try_from(secs).map_err(|_| CrossingError::DurationOverflow)?;
    Ok(Duration::new(secs, subsec_nanos as u32))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossingError {
    BeforeFirstMark,
    PastHorizon,
    EmptySummary,
    EmptyShape,
    InvalidAgeParams(String),
    InvalidShape(String),
    UnboundedAgeNotLast(u64),
    AgeBoundaryMismatch {
        previous: u64,
        current: u64,
    },
    PastAgeUnbounded(u64),
    KnownCrossingNotAfterAgeStart {
        known_epoch: u64,
        age_start_epoch: u64,
    },
    DurationOverflow,
    SlotOverflow,
}

impl fmt::Display for CrossingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BeforeFirstMark => f.write_str("crossing: time is before first mark"),
            Self::PastHorizon => f.write_str("crossing: slot/time past horizon"),
            Self::EmptySummary => f.write_str("crossing: summary must have at least one age"),
            Self::EmptyShape => f.write_str("crossing: shape must have at least one age"),
            Self::InvalidAgeParams(message) => write!(f, "crossing: invalid age params: {message}"),
            Self::InvalidShape(message) => write!(f, "crossing: invalid shape: {message}"),
            Self::UnboundedAgeNotLast(age_id) => {
                write!(f, "crossing: age {age_id} is unbounded but not last")
            }
            Self::AgeBoundaryMismatch { previous, current } => write!(
                f,
                "crossing: age {previous} end does not match age {current} start"
            ),
            Self::PastAgeUnbounded(age_id) => write!(f, "crossing: past age {age_id} is unbounded"),
            Self::KnownCrossingNotAfterAgeStart {
                known_epoch,
                age_start_epoch,
            } => write!(
                f,
                "crossing: known epoch {known_epoch} must be > age start epoch {age_start_epoch}"
            ),
            Self::DurationOverflow => f.write_str("crossing: duration overflow"),
            Self::SlotOverflow => f.write_str("crossing: slot overflow"),
        }
    }
}

impl std::error::Error for CrossingError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> AgeParams {
        AgeParams {
            epoch_size: 10,
            slot_length: Duration::from_secs(1),
            safe_zone_slots: 5,
            first_light_window: 0,
        }
    }

    fn shape() -> OghamShape {
        OghamShape {
            system_start: SystemTime::UNIX_EPOCH,
            ages: vec![OghamEntry {
                age_id: 0,
                age_name: "test".to_string(),
                min_major_version: 1,
                max_major_version: 1,
                params: params(),
                next_trigger: CrossingTrigger::not_in_this_rite(),
            }],
        }
    }

    #[test]
    fn shape_requires_final_not_in_this_rite() {
        let mut shape = shape();
        shape.ages[0].next_trigger = CrossingTrigger::at_version(2);
        assert!(matches!(
            shape.validate(),
            Err(CrossingError::InvalidShape(_))
        ));
    }

    #[test]
    fn known_crossing_builds_exact_epoch_boundary() {
        let shape = shape();
        let summary = build_ogham_summary(
            &shape,
            Vec::new(),
            AgeSummary {
                age_id: 0,
                start: OghamBound::ZERO,
                end: None,
                params: params(),
            },
            0,
            CrossingInfo::known(2),
        )
        .unwrap();

        assert_eq!(summary.current_age().unwrap().end.unwrap().slot, 20);
        assert_eq!(summary.slot_to_epoch(12).unwrap().epoch, 1);
    }

    #[test]
    fn slot_time_round_trip_stays_local() {
        let shape = shape();
        let summary = build_ogham_summary(
            &shape,
            Vec::new(),
            AgeSummary {
                age_id: 0,
                start: OghamBound::ZERO,
                end: None,
                params: params(),
            },
            0,
            CrossingInfo::known(3),
        )
        .unwrap();

        let time = summary.slot_to_time(7).unwrap();
        assert_eq!(time, SystemTime::UNIX_EPOCH + Duration::from_secs(7));
        assert_eq!(summary.time_to_slot(time).unwrap(), 7);
    }
}
