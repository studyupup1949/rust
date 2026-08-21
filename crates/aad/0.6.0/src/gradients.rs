use crate::variable::Variable;
pub struct Gradients<F>(pub(crate) Vec<F>);

impl<F: Copy> Gradients<F>
where
    Vec<F>: FromIterator<F>,
{
    #[must_use]
    pub fn get_gradient(&self, x: &Variable<F>) -> F {
        self.0[x.index]
    }
    #[must_use]
    pub fn get_gradients(&self, vars: &[Variable<F>]) -> Vec<F> {
        vars.iter().map(|var| self.get_gradient(var)).collect()
    }
}
