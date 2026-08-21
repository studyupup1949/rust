use crate::variable::Variable;
pub struct Gradients<F>(pub(crate) Vec<F>);

impl<F: Copy> Gradients<F>
where
    Vec<F>: FromIterator<F>,
{
    #[inline]
    #[must_use]
    pub fn get_gradient(&self, x: &Variable<F>) -> F {
        self.0[x.index.unwrap().0]
    }

    #[inline]
    #[must_use]
    pub fn get_gradients<const N: usize>(&self, vars: &[Variable<F>; N]) -> [F; N] {
        std::array::from_fn(|i| self.get_gradient(&vars[i]))
    }

    #[inline]
    #[must_use]
    pub fn get_gradients_vec(&self, vars: &[Variable<F>]) -> Vec<F> {
        vars.iter().map(|var| self.get_gradient(var)).collect()
    }
}
