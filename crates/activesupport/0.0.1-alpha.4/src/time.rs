pub use chrono::{DateTime, Duration, TimeZone, Timelike, Utc};

pub trait TimeDuration {
    fn in_milliseconds(&self) -> i32;

    fn seconds(&self) -> Duration;

    #[inline]
    fn second(&self) -> Duration {
        self.seconds()
    }

    fn minutes(&self) -> Duration;
    #[inline]
    fn minute(&self) -> Duration {
        self.minutes()
    }

    fn hours(&self) -> Duration;
    #[inline]
    fn hour(&self) -> Duration {
        self.hours()
    }

    fn days(&self) -> Duration;
    #[inline]
    fn day(&self) -> Duration {
        self.days()
    }

    fn weeks(&self) -> Duration;
    #[inline]
    fn week(&self) -> Duration {
        self.weeks()
    }

    fn fortnights(&self) -> Duration;
    #[inline]
    fn fortnight(&self) -> Duration {
        self.fortnights()
    }
}

// TODO: use macro to define this for all primitive integers
impl TimeDuration for i32 {
    #[inline]
    fn seconds(&self) -> Duration {
        Duration::seconds((*self).into())
    }

    #[inline]
    fn minutes(&self) -> Duration {
        Duration::minutes((*self).into())
    }

    #[inline]
    fn hours(&self) -> Duration {
        Duration::hours((*self).into())
    }

    #[inline]
    fn days(&self) -> Duration {
        Duration::days((*self).into())
    }

    #[inline]
    fn weeks(&self) -> Duration {
        Duration::weeks((*self).into())
    }

    #[inline]
    fn fortnights(&self) -> Duration {
        Duration::weeks((2 * self).into())
    }

    #[inline]
    fn in_milliseconds(&self) -> i32 {
        (*self) * 1000
    }
    // TODO: months, years
}

pub trait TimeRange {
    #[allow(clippy::wrong_self_convention)]
    fn from_now(&self) -> Option<chrono::DateTime<Utc>>;
    #[inline]
    fn since(&self) -> Option<chrono::DateTime<Utc>> {
        self.from_now()
    }

    #[inline]
    fn after(&self) -> Option<chrono::DateTime<Utc>> {
        self.from_now()
    }

    fn ago(&self) -> Option<DateTime<Utc>>;

    #[inline]
    fn until(&self) -> Option<DateTime<Utc>> {
        self.ago()
    }

    #[inline]
    fn before(&self) -> Option<DateTime<Utc>> {
        self.ago()
    }
}

impl TimeRange for Duration {
    fn from_now(&self) -> Option<DateTime<Utc>> {
        Utc::now().checked_add_signed(self.to_owned())
    }

    fn ago(&self) -> Option<DateTime<Utc>> {
        Utc::now().checked_sub_signed(self.to_owned())
    }
}

pub trait TimeCalculation {
    fn beginning_of_hour(&self) -> Option<DateTime<Utc>>;
    fn end_of_hour(&self) -> Option<DateTime<Utc>>;
    fn beginning_of_minute(&self) -> Option<DateTime<Utc>>;
    fn end_of_minute(&self) -> Option<DateTime<Utc>>;
}

impl TimeCalculation for DateTime<Utc> {
    // TODO: beginning_of_day, end_of_day
    fn beginning_of_hour(&self) -> Option<DateTime<Utc>> {
        let a: i64 = self.timestamp_nanos() % 3_600_000_000_000;
        self.to_owned().checked_sub_signed(Duration::nanoseconds(a))
    }

    fn end_of_hour(&self) -> Option<DateTime<Utc>> {
        self.beginning_of_hour().and_then(|f| {
            f.checked_add_signed(Duration::nanoseconds((3600 - 1) * 1000000000 + 999999999))
        })
    }

    fn beginning_of_minute(&self) -> Option<DateTime<Utc>> {
        let a: i64 = self.timestamp_nanos() % 60_000_000_000;
        self.to_owned().checked_sub_signed(Duration::nanoseconds(a))
    }

    fn end_of_minute(&self) -> Option<DateTime<Utc>> {
        self.beginning_of_minute().and_then(|f| {
            f.checked_add_signed(Duration::nanoseconds((60 - 1) * 1000000000 + 999999999))
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn duration_works() {
        assert_eq!(60.seconds(), Duration::seconds((60).into()));
        assert_eq!(1.second(), Duration::seconds((1).into()));
        assert_eq!(60.minutes(), Duration::minutes((60).into()));
        assert_eq!(1.minute(), Duration::minutes((1).into()));
        assert_eq!(60.hours(), Duration::hours((60).into()));
        assert_eq!(1.hour(), Duration::hours((1).into()));
        assert_eq!(60.days(), Duration::days((60).into()));
        assert_eq!(1.day(), Duration::days((1).into()));
        assert_eq!(60.weeks(), Duration::weeks((60).into()));
        assert_eq!(1.week(), Duration::weeks((1).into()));
        assert_eq!(60.fortnights(), Duration::weeks((120).into()));
        assert_eq!(1.fortnight(), Duration::weeks((2).into()));
    }

    #[test]
    fn range_works() {
        assert!(1.day().from_now() <= Utc::now().checked_add_signed(1.day().to_owned()));
        assert!(2.weeks().from_now() <= Utc::now().checked_add_signed(2.weeks().to_owned()));
        assert!(
            (4.days() + 5.weeks()).from_now()
                <= Utc::now().checked_add_signed((4.days() + 5.weeks()).to_owned())
        );

        assert!(1.day().ago() <= Utc::now().checked_sub_signed(1.day().to_owned()));
        assert!(2.weeks().ago() <= Utc::now().checked_sub_signed(2.weeks().to_owned()));
        assert!(
            (4.days() + 5.weeks()).ago()
                <= Utc::now().checked_sub_signed((4.days() + 5.weeks()).to_owned())
        );
    }

    #[test]
    fn in_milliseconds_works() {
        assert!(1.in_milliseconds() <= 1000);
    }

    #[test]
    fn beginning_of_hour_works() {
        let a = Utc::now().beginning_of_hour().unwrap();
        assert_eq!(a.second(), 0);
        assert_eq!(a.nanosecond(), 0);
    }

    #[test]
    fn end_of_hour_works() {
        let a = Utc::now().end_of_hour().unwrap();
        assert_eq!(a.second() % 60, 59);
        assert_eq!(a.nanosecond(), 999999999);
    }

    #[test]
    fn beginning_of_minute_works() {
        let a = Utc::now().beginning_of_minute().unwrap();
        assert_eq!(a.second(), 0);
        assert_eq!(a.nanosecond(), 0);
    }

    #[test]
    fn end_of_minute_works() {
        let a = Utc::now().end_of_minute().unwrap();
        assert_eq!(a.second(), 59);
        assert_eq!(a.nanosecond(), 999999999);
    }
}
