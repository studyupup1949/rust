use crate::ledger::eras::{CrossingError, EpochInfo, OghamSummary};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkTick {
    pub slot: u64,
    pub slot_start: SystemTime,
    pub age: u64,
    pub age_slot: u64,
    pub is_age_start: bool,
    pub slots_until_age: u64,
}

pub trait MarkTimeProvider {
    fn slot_to_time(&self, slot: u64) -> Result<SystemTime, CrossingError>;
    fn time_to_slot(&self, time: SystemTime) -> Result<u64, CrossingError>;
    fn slot_to_epoch(&self, slot: u64) -> Result<EpochInfo, CrossingError>;
}

impl MarkTimeProvider for OghamSummary {
    fn slot_to_time(&self, slot: u64) -> Result<SystemTime, CrossingError> {
        OghamSummary::slot_to_time(self, slot)
    }

    fn time_to_slot(&self, time: SystemTime) -> Result<u64, CrossingError> {
        OghamSummary::time_to_slot(self, time)
    }

    fn slot_to_epoch(&self, slot: u64) -> Result<EpochInfo, CrossingError> {
        OghamSummary::slot_to_epoch(self, slot)
    }
}

pub struct SunwiseClock<P> {
    provider: P,
    now: Box<dyn Fn() -> SystemTime + Send + Sync>,
}

impl<P> SunwiseClock<P>
where
    P: MarkTimeProvider,
{
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            now: Box::new(SystemTime::now),
        }
    }

    pub fn with_now(mut self, now: impl Fn() -> SystemTime + Send + Sync + 'static) -> Self {
        self.now = Box::new(now);
        self
    }

    pub fn current_slot(&self) -> Result<u64, CrossingError> {
        self.provider.time_to_slot((self.now)())
    }

    pub fn current_age(&self) -> Result<EpochInfo, CrossingError> {
        let slot = self.current_slot()?;
        self.provider.slot_to_epoch(slot)
    }

    pub fn age_for_slot(&self, slot: u64) -> Result<EpochInfo, CrossingError> {
        self.provider.slot_to_epoch(slot)
    }

    pub fn slot_to_time(&self, slot: u64) -> Result<SystemTime, CrossingError> {
        self.provider.slot_to_time(slot)
    }

    pub fn next_slot_time(&self) -> Result<SystemTime, CrossingError> {
        let slot = self.current_slot()?;
        self.provider.slot_to_time(slot + 1)
    }

    pub fn time_until_slot(&self, slot: u64) -> Result<Duration, CrossingError> {
        let target = self.provider.slot_to_time(slot)?;
        Ok(target
            .duration_since((self.now)())
            .unwrap_or_else(|err| err.duration()))
    }

    pub fn tick_at(&self, time: SystemTime) -> Result<MarkTick, CrossingError> {
        let slot = self.provider.time_to_slot(time)?;
        let epoch = self.provider.slot_to_epoch(slot)?;
        let age_slot = slot - epoch.start_slot;
        Ok(MarkTick {
            slot,
            slot_start: self.provider.slot_to_time(slot)?,
            age: epoch.epoch,
            age_slot,
            is_age_start: age_slot == 0,
            slots_until_age: epoch.length_in_slots - age_slot,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::eras::{
        build_ogham_summary, AgeParams, AgeSummary, CrossingInfo, CrossingTrigger, OghamBound,
        OghamEntry, OghamShape,
    };

    fn summary() -> OghamSummary {
        let params = AgeParams {
            epoch_size: 10,
            slot_length: Duration::from_secs(1),
            safe_zone_slots: 0,
            first_light_window: 0,
        };
        let shape = OghamShape {
            system_start: SystemTime::UNIX_EPOCH,
            ages: vec![OghamEntry {
                age_id: 0,
                age_name: "age".to_string(),
                min_major_version: 1,
                max_major_version: 1,
                params,
                next_trigger: CrossingTrigger::not_in_this_rite(),
            }],
        };
        build_ogham_summary(
            &shape,
            Vec::new(),
            AgeSummary {
                age_id: 0,
                start: OghamBound::ZERO,
                end: None,
                params,
            },
            0,
            CrossingInfo::impossible(),
        )
        .unwrap()
    }

    #[test]
    fn tick_at_reports_age_boundary_information() {
        let clock = SunwiseClock::new(summary());
        let tick = clock
            .tick_at(SystemTime::UNIX_EPOCH + Duration::from_secs(10))
            .unwrap();
        assert_eq!(tick.slot, 10);
        assert_eq!(tick.age, 1);
        assert_eq!(tick.age_slot, 0);
        assert!(tick.is_age_start);
    }
}
