//! The crate `calendar` provides types and methods for income and settlement dates
//! adjustment.


use chrono::{Datelike, Duration, NaiveDate, Weekday};

/// A `BusinessDayConvetion` represents the method of date rolling in case
/// it falls on a non-business day.
#[derive(Debug)]
#[non_exhaustive]
pub enum BusinessDayConvetion {
    /// The adjusted date will be the first business day following the unadjusted date.
    Following,
    /// The adjusted date will be the first business day following the unadjusted date,
    /// unless it falls in the next calendar month - then the first preceding business day
    /// is the adjusted date.
    ModifiedFollowiing,
    /// The adjusted date will be the first business day preceding the unadjusted date.
    Preceding,
    /// The adjusted date will be the first business day preceding the unadjusted date,
    /// unless it falls in the previous calendar month - then the first following business day
    /// is the adjusted date.
    ModifiedPreceding,
    /// No date adjustment is made.
    NoAdjustment,
}

/// `BusinessCalendar` trait allows implementation of bank holiday calendars,
/// which are used for accrual calculation date rolling.
///
/// The trait is built upon [`chrono::NaiveDate`] type - as such it follows
/// the `ISO 8601` standard.
pub trait BusinessCalendar {
    /// Checks whether the date is a bank holiday.
    fn is_holiday(&self, day: NaiveDate) -> bool;

    /// Checks whether the date falls on a weekend.
    /// Default implementation assumes that weekend consists of Saturday and Sunday,
    /// which is not true for all countries.
    fn is_weekend(&self, day: NaiveDate) -> bool {
        !(day.weekday() == Weekday::Sat) && !(day.weekday() == Weekday::Sun)
    }

    /// Check whether the date is a business day.
    /// By default, it is assumed that if the day does not fall
    /// on weekend and is not a bank holiday, then it's a business day.
    fn is_business(&self, day: NaiveDate) -> bool {
        !self.is_holiday(day) && !self.is_weekend(day)
    }

    /// Calculates the adjusted date using the `following` convention and returns it in a form of
    /// `Option<NaiveDate>` enum.
    ///
    /// If the supplied date is a business date, it is returned with no adjustment.
    /// Otherwise, the adjusted date will be the first business day following the unadjusted date.
    ///
    /// Returns `None` if no such business day exist.
    fn following(&self, mut day: NaiveDate) -> Option<NaiveDate> {
        if self.is_business(day) {
            return Some(day);
        }

        while day < chrono::naive::MAX_DATE {
            day += Duration::days(1);
            if self.is_business(day) {
                return Some(day);
            }
        }

        None
    }


    /// Calculates the adjusted date using the `modified following` convention and returns it in a form of
    /// `Option<NaiveDate>` enum.
    ///
    /// If the supplied date is a business date, it is returned with no adjustment.
    /// Otherwise, the adjusted date will be the first business day following the unadjusted date,
    /// unless it falls in the next month. In such case, the first preceding business day is returned.
    /// 
    /// Returns `None` if no such business day exist.    
    fn modified_following(&self, mut day: NaiveDate) -> Option<NaiveDate> {
        let unadjusted = day;
        if self.is_business(day) {
            return Some(day);
        }

        while day < chrono::naive::MAX_DATE && day.month() == unadjusted.month() {
            day += Duration::days(1);
            if day.month() != unadjusted.month() {
                break;
            }
        }
        day = unadjusted;
        while day > chrono::naive::MIN_DATE {
            day -= Duration::days(1);
            if self.is_business(day) {
                return Some(day);
            }
        }
        None
    }


    /// Calculates the adjusted date using the `preceding` convention and returns it in a form of
    /// `Option<NaiveDate>` enum.
    ///
    /// If the supplied date is a business date, it is returned with no adjustment.
    /// Otherwise, the adjusted date will be the first business day before the unadjusted date.
    ///
    /// Returns `None` if no such business day exist.
    fn preceding(&self, mut day: NaiveDate) -> Option<NaiveDate> {
        if self.is_business(day) {
            return Some(day);
        }

        while day < chrono::naive::MIN_DATE {
            day -= Duration::days(1);
            if self.is_business(day) {
                return Some(day);
            }
        }

        None
    }


    /// Calculates the adjusted date using the `following` convention and returns it in a form of
    /// `Option<NaiveDate>` enum.
    ///
    /// If the supplied date is a business date, it is returned with no adjustment.
    /// Otherwise, the adjusted date will be the first business day before the unadjusted date,
    /// unless it falls in the previous month. In such case, the next following bussiness day is
    /// the adjusted date.
    ///
    /// Returns `None` if no such business day exist.
    fn modified_preceding(&self, mut day: NaiveDate) -> Option<NaiveDate> {
        let unadjusted = day;
        if self.is_business(day) {
            return Some(day);
        }

        while day > chrono::naive::MIN_DATE{
            day -= Duration::days(1);
            if day.month() != unadjusted.month() {
                break;
            }            
            if self.is_business(day) {
                return Some(day);
            }
        }

        day = unadjusted;
        while day < chrono::naive::MAX_DATE {
            day += Duration::days(1);
            if self.is_business(day) {
                return Some(day);
            }
        }
        None
    }
}
