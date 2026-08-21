//! ```rust
//! use activesupport::time::{Duration, TimeDuration, TimeRange, Utc, TimeCalculation, Timelike};
//!
//! assert_eq!(60.seconds(), Duration::seconds((60).into()));
//! assert_eq!(1.second(), Duration::seconds((1).into()));
//! assert_eq!(60.minutes(), Duration::minutes((60).into()));
//! assert_eq!(1.minute(), Duration::minutes((1).into()));
//! assert_eq!(60.hours(), Duration::hours((60).into()));
//! assert_eq!(1.hour(), Duration::hours((1).into()));
//! assert_eq!(60.days(), Duration::days((60).into()));
//! assert_eq!(1.day(), Duration::days((1).into()));
//! assert_eq!(60.weeks(), Duration::weeks((60).into()));
//! assert_eq!(1.week(), Duration::weeks((1).into()));
//! assert_eq!(60.fortnights(), Duration::weeks((120).into()));
//! assert_eq!(1.fortnight(), Duration::weeks((2).into()));
//!
//! assert!(1.day().from_now() <= Utc::now().checked_add_signed(1.day().to_owned()));
//! assert!(2.weeks().from_now() <= Utc::now().checked_add_signed(2.weeks().to_owned()));
//! assert!(
//!     (4.days() + 5.weeks()).from_now()
//!         <= Utc::now().checked_add_signed((4.days() + 5.weeks()).to_owned())
//! );
//!
//! assert!(1.day().ago() <= Utc::now().checked_sub_signed(1.day().to_owned()));
//! assert!(2.weeks().ago() <= Utc::now().checked_sub_signed(2.weeks().to_owned()));
//! assert!(
//!     (4.days() + 5.weeks()).ago()
//!         <= Utc::now().checked_sub_signed((4.days() + 5.weeks()).to_owned())
//! );
//!
//! assert!(1.in_milliseconds() <= 1000);
//!
//! let a = Local::now().beginning_of_hour().unwrap();
//! assert_eq!(a.second(), 0);
//! assert_eq!(a.nanosecond(), 0);
//!
//! let a = Utc::now().end_of_hour().unwrap();
//! assert_eq!(a.second(), 59);
//! assert_eq!(a.nanosecond(), 999999999);
//!
//! let a = Local::now().end_of_hour().unwrap();
//! assert_eq!(a.second(), 59);
//! assert_eq!(a.nanosecond(), 999999999);
//!
//! let a = Utc::now().beginning_of_minute().unwrap();
//! assert_eq!(a.second(), 0);
//! assert_eq!(a.nanosecond(), 0);
//!
//! let a = Local::now().beginning_of_minute().unwrap();
//! assert_eq!(a.second(), 0);
//! assert_eq!(a.nanosecond(), 0);
//!
//! let a = Utc::now().end_of_minute().unwrap();
//! assert_eq!(a.second(), 59);
//! assert_eq!(a.nanosecond(), 999999999);
//!
//! let a = Local::now().end_of_minute().unwrap();
//! assert_eq!(a.second(), 59);
//! assert_eq!(a.nanosecond(), 999999999);
//!
//! let a = Utc::now().beginning_of_day().unwrap();
//! assert_eq!(a.hour(), 0);
//! assert_eq!(a.second(), 0);
//! assert_eq!(a.nanosecond(), 0);
//!
//! let a = Local::now().beginning_of_day().unwrap();
//! assert_eq!(a.hour(), 0);
//! assert_eq!(a.second(), 0);
//! assert_eq!(a.nanosecond(), 0);
//!
//! let a = Utc::now().end_of_day().unwrap();
//! assert_eq!(a.hour(), 23);
//! assert_eq!(a.second(), 59);
//! assert_eq!(a.nanosecond(), 999999999);
//!
//! let a = Local::now().end_of_day().unwrap();
//! assert_eq!(a.hour(), 23);
//! assert_eq!(a.second(), 59);
//! assert_eq!(a.nanosecond(), 999999999);
//! ```
pub mod time;
