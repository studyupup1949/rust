use crate::var::Var;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
pub struct Grads(pub(crate) Vec<f64>);

impl Grads {
    pub fn get_one(&self, x: &Var) -> f64 {
        unsafe { *self.0.get_unchecked(x.idx) }
    }
    pub fn get(&self, vars: &[Var]) -> Vec<f64> {
        vars.par_iter().map(|var| self.get_one(var)).collect()
    }
}
