//! Options configure each phase of a factorization.
//!
//! The option types mirror Accelerate's fields and defaults. A zeroed struct is not equivalent:
//! the numeric tolerances have non-zero defaults.

use accelerate_sparse_sys as sys;

use crate::{FactorizationKind, scalar::Scalar};

/// Selects the fill-reducing ordering used by the symbolic phase.
///
/// Every ordering reaches the same solution; they trade analysis time against how much fill the
/// factor picks up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum OrderMethod {
    /// Let Accelerate choose.
    #[default]
    Default,
    /// Approximate minimum degree.
    Amd,
    /// Nested dissection.
    Metis,
    /// Column approximate minimum degree, an ordering for the normal equations. Applies only to
    /// [`FactorizationKind::Qr`] and [`FactorizationKind::CholeskyAtA`]; pairing it with a
    /// symmetric factorization returns an input error from [`SymbolicFactorization::new`], because
    /// Accelerate does not reject it but instead **spins forever** inside the numeric phase once
    /// the matrix is large enough to reach its supernodal path (observed).
    ///
    /// [`SymbolicFactorization::new`]: crate::SymbolicFactorization::new
    Colamd,
}

impl OrderMethod {
    fn to_raw(self) -> u8 {
        match self {
            Self::Default => sys::ACCSP_ORDER_DEFAULT,
            Self::Amd => sys::ACCSP_ORDER_AMD,
            Self::Metis => sys::ACCSP_ORDER_METIS,
            Self::Colamd => sys::ACCSP_ORDER_COLAMD,
        }
    }

    /// Whether this ordering is safe to use with `kind`. Only COLAMD is restricted, to the
    /// normal-equations factorizations it was designed for; every other ordering applies to every
    /// kind.
    pub(crate) fn applies_to(self, kind: FactorizationKind) -> bool {
        !matches!(self, Self::Colamd)
            || matches!(kind, FactorizationKind::Qr | FactorizationKind::CholeskyAtA)
    }
}

/// Configures the symbolic phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SymbolicOptions {
    order_method: Option<OrderMethod>,
}

impl SymbolicOptions {
    /// Options that select Accelerate's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the fill-reducing ordering.
    pub fn order_method(mut self, order_method: OrderMethod) -> Self {
        self.order_method = Some(order_method);
        self
    }

    /// The ordering the caller chose, if any, so the symbolic phase can reject a pairing that
    /// would not terminate.
    pub(crate) fn chosen_order(&self) -> Option<OrderMethod> {
        self.order_method
    }

    /// Builds the raw struct, or `None` when nothing was customised so the defaults can be
    /// selected by passing no options at all.
    pub(crate) fn to_raw(self) -> Option<sys::accsp_symbolic_options> {
        let order_method = self.order_method?;
        let mut raw = sys::accsp_symbolic_options {
            control: 0,
            order_method: 0,
        };
        // SAFETY: `raw` is a live, writable local.
        unsafe { sys::accsp_default_symbolic_options(&mut raw) };
        raw.order_method = order_method.to_raw();
        Some(raw)
    }
}

/// Configures the numeric phase.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NumericOptions {
    pivot_tolerance: Option<f64>,
    zero_tolerance: Option<f64>,
}

impl NumericOptions {
    /// Options that select Accelerate's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the relative threshold below which a pivot is rejected.
    pub fn pivot_tolerance(mut self, tolerance: f64) -> Self {
        self.pivot_tolerance = Some(tolerance);
        self
    }

    /// Sets the magnitude below which a pivot is treated as zero.
    ///
    /// **Observed:** This applies to pivots produced by elimination, but not to equal-sized entries
    /// on the original diagonal. [`Factorization::inertia`](crate::Factorization::inertia) reports
    /// how many pivots were treated as zero.
    pub fn zero_tolerance(mut self, tolerance: f64) -> Self {
        self.zero_tolerance = Some(tolerance);
        self
    }

    /// Builds the raw struct, or `None` when nothing was customised.
    ///
    /// Anything left unset keeps Accelerate's default, read from the framework rather than
    /// restated here.
    pub(crate) fn to_raw<T: Scalar>(self) -> Option<sys::accsp_numeric_options> {
        if self.pivot_tolerance.is_none() && self.zero_tolerance.is_none() {
            return None;
        }
        let mut raw = sys::accsp_numeric_options {
            control: 0,
            scaling_method: 0,
            pivot_tolerance: 0.0,
            zero_tolerance: 0.0,
        };
        // SAFETY: `raw` is a live, writable local.
        unsafe { T::default_numeric_options(&mut raw) };
        if let Some(tolerance) = self.pivot_tolerance {
            raw.pivot_tolerance = tolerance;
        }
        if let Some(tolerance) = self.zero_tolerance {
            raw.zero_tolerance = tolerance;
        }
        Some(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults<T: Scalar>() -> sys::accsp_numeric_options {
        let mut out = sys::accsp_numeric_options {
            control: 0,
            scaling_method: 0,
            pivot_tolerance: 0.0,
            zero_tolerance: 0.0,
        };
        // SAFETY: `out` is a live, writable local.
        unsafe { T::default_numeric_options(&mut out) };
        out
    }

    #[test]
    fn unset_options_pass_none_so_accelerate_picks_its_own_defaults() {
        assert!(NumericOptions::new().to_raw::<f64>().is_none());
        assert!(NumericOptions::new().to_raw::<f32>().is_none());
        assert!(SymbolicOptions::new().to_raw().is_none());
    }

    #[test]
    fn a_set_tolerance_is_applied_and_the_rest_stay_at_the_non_zero_defaults() {
        let d = defaults::<f64>();
        // The guarantee under test: the untouched fields keep Accelerate's non-zero defaults
        // rather than the zeros of a bare struct, which for the tolerances is a different matrix.
        assert_ne!(d.pivot_tolerance, 0.0);
        assert_ne!(d.zero_tolerance, 0.0);

        let raw = NumericOptions::new()
            .pivot_tolerance(0.5)
            .to_raw::<f64>()
            .expect("a customised field yields Some");
        assert_eq!(raw.pivot_tolerance, 0.5);
        assert_eq!(raw.zero_tolerance, d.zero_tolerance);
        assert_eq!(raw.control, d.control);
        assert_eq!(raw.scaling_method, d.scaling_method);

        let raw = NumericOptions::new()
            .zero_tolerance(1e-3)
            .to_raw::<f64>()
            .unwrap();
        assert_eq!(raw.zero_tolerance, 1e-3);
        assert_eq!(raw.pivot_tolerance, d.pivot_tolerance);
    }

    #[test]
    fn the_untouched_defaults_are_read_for_the_element_type_asked_for() {
        // Accelerate's single- and double-precision pivot defaults differ, so a field left unset
        // must be filled from the type in hand rather than borrowed from the other.
        let (d, f) = (defaults::<f64>(), defaults::<f32>());
        assert_ne!(
            d.pivot_tolerance, f.pivot_tolerance,
            "test premise: the per-type defaults differ"
        );

        let raw = NumericOptions::new()
            .zero_tolerance(1e-3)
            .to_raw::<f32>()
            .unwrap();
        assert_eq!(raw.pivot_tolerance, f.pivot_tolerance);
    }
}
