use crate::variable::Variable;
use std::mem::MaybeUninit;

#[derive(Debug, PartialEq)]
pub enum GradientError {
    MissingIndex,
    OutOfBounds(usize, usize),
}

pub struct Gradients<F>(pub(crate) Vec<F>);

impl<F: Copy> Gradients<F>
where
    Vec<F>: FromIterator<F>,
{
    #[inline]
    /// Returns the gradient for the given variable.
    ///
    /// # Arguments
    ///
    /// * `x` - Variable to get gradient for
    ///
    /// # Returns
    ///
    /// * `Result<F, GradientError>` - Gradient if successful, error if gradient cannot be found
    ///
    /// # Errors
    ///
    /// Returns `GradientError::MissingIndex` if the variable does not have an index
    /// Returns `GradientError::OutOfBounds` if the variable's index is out of bounds
    pub fn get_gradient(&self, x: &Variable<F>) -> Result<F, GradientError> {
        let index = x.index.ok_or(GradientError::MissingIndex)?;
        let idx = index.0;
        self.0
            .get(idx)
            .copied()
            .ok_or(GradientError::OutOfBounds(idx, self.0.len()))
    }

    #[inline]
    /// Returns an array of gradients for the given array of variables.
    ///
    /// # Arguments
    ///
    /// * `vars` - Array of variables to get gradients for
    ///
    /// # Returns
    ///
    /// * `Result<[F; N], GradientError>` - Array of gradients if successful, error if any variable's gradient cannot be found
    ///   Returns an array of gradients for the given array of variables.
    ///
    /// # Arguments
    ///
    /// * `vars` - Array of variables to get gradients for
    ///
    /// # Returns
    ///
    /// * `Result<[F; N], GradientError>` - Array of gradients if successful, error if any variable's gradient cannot be found
    ///
    /// # Errors
    ///
    /// Returns `GradientError::MissingIndex` if any variable does not have an index
    /// Returns `GradientError::OutOfBounds` if any variable's index is out of bounds
    pub fn get_gradients<const N: usize>(
        &self,
        vars: &[Variable<F>; N],
    ) -> Result<[F; N], GradientError> {
        let mut arr: [MaybeUninit<F>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        for (i, var) in vars.iter().enumerate() {
            arr[i] = MaybeUninit::new(self.get_gradient(var)?);
        }

        Ok(unsafe { std::mem::transmute_copy(&arr) })
    }

    #[inline]
    /// Returns an iterator over gradients for the given slice of variables.
    ///
    /// # Arguments
    ///
    /// * `vars` - Slice of variables to get gradients for
    ///
    /// # Returns
    ///
    /// * `impl Iterator<Item = Result<F, GradientError>>` - Iterator yielding gradients or errors
    ///
    /// # Errors
    ///
    /// Each iteration may return:
    /// * `GradientError::MissingIndex` if a variable does not have an index
    /// * `GradientError::OutOfBounds` if a variable's index is out of bounds
    pub fn get_gradients_iter(
        &self,
        vars: &[Variable<F>],
    ) -> impl Iterator<Item = Result<F, GradientError>> {
        vars.iter().map(|var| self.get_gradient(var))
    }
}
