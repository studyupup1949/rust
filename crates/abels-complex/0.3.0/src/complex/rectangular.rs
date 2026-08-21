pub type Complex32 = Complex<f32>;
pub type Complex64 = Complex<f64>;

/// Creates a complex number in rectangular form.
#[inline(always)]
#[must_use]
pub const fn complex<FT>(re: FT, im: FT) -> Complex<FT> {
    Complex::new(re, im)
}

/// A complex number in rectangular form.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Complex<FT> {
    pub re: FT,
    pub im: FT,
}

impl<FT> Complex<FT> {
    /// Creates a complex number.
    pub const fn new(re: FT, im: FT) -> Self {
        Self { re, im }
    }
}
