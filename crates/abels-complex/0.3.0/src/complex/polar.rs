pub type ComplexPolar32 = ComplexPolar<f32>;
pub type ComplexPolar64 = ComplexPolar<f64>;

/// Creates a complex number in polar form.
#[inline(always)]
#[must_use]
pub const fn complex_polar<FT>(abs: FT, arg: FT) -> ComplexPolar<FT> {
    ComplexPolar::new(abs, arg)
}

/// A complex number in polar form.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct ComplexPolar<FT> {
    pub abs: FT,
    pub arg: FT,
}

impl<FT> ComplexPolar<FT> {
    /// Creates a complex number.
    pub const fn new(abs: FT, arg: FT) -> Self {
        Self { abs, arg }
    }
}
