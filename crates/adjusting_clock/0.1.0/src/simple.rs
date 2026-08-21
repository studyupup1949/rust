//! Simple types for typical use cases.

use crate::{Clock, ConstantRate};

pub type AdjustingClock = Clock<ConstantRate>;
