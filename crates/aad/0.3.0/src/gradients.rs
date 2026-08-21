use crate::variable::Variable;
pub struct Gradients(pub(crate) Vec<f64>);

impl Gradients {
    pub fn get_gradient(&self, x: &Variable) -> f64 {
        self.0[x.index]
    }
    pub fn get_gradients(&self, vars: &[Variable]) -> Vec<f64> {
        vars.iter().map(|var| self.get_gradient(var)).collect()
    }
}
