//! Manipulate the probabilities of working on different solutions.
//!
//! A portion of the bees in an artificial bee colony are tasked with observing
//! the dedicated workers, and doing extra work on promising solutions. To
//! enable this, the solutions' fitnesses are gathered as a `Vec<f64>`. A
//! [`FitnessScaler`](type.FitnessScaler.html) is then run on the fitnesses to
//! get weighting factors, and a solution is chosen with likelihood
//! proportionate to its scaled fitness. This is expressed as:
//!
//! <center>P(*i*) = *scaled*<sub>*i*</sub>
//! / ∑<sub>*j* = 1 … N</sub> *scaled*<sub>*j*</sub></center>
//!
//! By default, [proportionate](fn.proportionate.html) scaling is used.
//!
//! To construct a custom FitnessScaler, you can use the
//! [`scale_by!`](../macro.scale_by!.html) macro provided.
//!
//! # Examples
//!
//! Several constructors for scaling functions are available in this module.
//! However, users may also implement custom scaling functions. These can
//! exaggerate differences in fitness:
//!
//! ```
//! # #[macro_use] extern crate abc; fn main() {
//! scale_by!(fitnesses => {
//!     // Square the fitnesses.
//!     fitnesses.iter().map(|fitness| fitness.powf(2f64)).collect()
//! });
//! # }
//! ```

pub use std::sync::Arc;
use std::ops::Deref;

pub type Callback = Fn(Vec<f64>) -> Vec<f64> + Send + Sync + 'static;

/// Map a vector of fitnesses to a vector of scaled fitnesses.
///
/// For more information, see the [module docs](index.html).
pub struct FitnessScaler {
    arc: Arc<Callback>,
}

impl FitnessScaler {
    pub fn new(arc: Arc<Callback>) -> FitnessScaler {
        FitnessScaler {
            arc: arc
        }
    }
}

impl Clone for FitnessScaler {
    fn clone(&self) -> Self {
        FitnessScaler {
            arc: self.arc.clone()
        }
    }
}

impl Deref for FitnessScaler {
    type Target = Callback;

    fn deref(&self) -> &Self::Target {
        self.arc.deref()
    }
}

#[macro_export]
/// Construct a custom FitnessScaler.
///
/// The body of this macro includes a name and a formula. These will be used to
/// construct a closure that always takes a `Vec<f64>` and returns a `Vec<f64>`,
/// and that always uses `move`.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate abc;
///
/// # fn main() {
/// scale_by!(fits => fits.iter().map(|f| -f).collect());
/// # }
/// ```
///
/// As with a closure, the formula can be a block:
///
/// ```
/// # #[macro_use] extern crate abc; fn main() {
/// scale_by!(fitnesses => {
///     // Bring all fitnesses halfway towards the mean.
///     let length = fitnesses.len() as f64;
///     let mean = fitnesses.iter().fold(0f64, |total, x| total + x) / length;
///     fitnesses.iter().map(|x| x - (x - mean) / 2f64).collect()
/// });
/// # }
/// ```
macro_rules! scale_by {
    ($name:ident => $formula:expr) => {{
        let arc = $crate::scaling::Arc::new(move|$name: Vec<f64>| $formula);
        $crate::scaling::FitnessScaler::new(arc)
    }}
}

/// Choose solutions in direct proportion to their fitness.
///
/// scaled<sub>*i*</sub> = fitness<sub>*i*</sub>
pub fn proportionate() -> FitnessScaler {
    scale_by!(fitnesses => fitnesses)
}

/// Choose more fit solutions exponentially more often.
///
/// scaled<sub>*i*</sub> = fitness<sub>*i*</sub><sup>*k*</sup>
pub fn power(k: f64) -> FitnessScaler {
    scale_by!(fitnesses => fitnesses.iter().map(|f| f.powf(k)).collect())
}

/// Choose solutions according to their rank.
///
/// Rather than use the fitness directly, this formula ranks the N solutions
/// 1 to N, in ascending order of fitness, then chooses in proportion to the
/// value of the rank. So, the solution with rank 6 will be chosen twice as
/// often as the solution with rank 3, and three times as often as the solution
/// with rank 2.
///
/// scaled<sub>*i*</sub> = rank<sub>*i*</sub>
pub fn rank() -> FitnessScaler {
    // power_rank is be implemented on its own because composing scaling
    // functions involves allocating extra vectors, which we'd like to avoid.
    power_rank(1f64)
}

/// Choose solutions according to their rank, raised to a certain power.
///
/// This scaling formula was proposed by Yudong Zhang et al for the
/// Fitness-Scaling Chaotic ABC in the 2013 volume of *Mathematical Problems
/// in Engineering*. Conceptually, it composes the [power](fn.power.html) and
/// [rank](fn.rank.html) scaling techniques.
///
/// scaled<sub>*i*</sub> = rank<sub>*i*</sub><sup>*k*</sup>
///
/// As with rank scaling, rank<sub>*i*</sub> starts with 1 for the least fit,
/// and continues up to N for the most fit.
pub fn power_rank(k: f64) -> FitnessScaler {
    scale_by!(fitnesses => {
        // Pair each fitness with its index, so that we can remember which goes
        // where after sorting.
        let mut with_indices = fitnesses.iter().enumerate().collect::<Vec<_>>();

        // Sort by fitness, ascending. After this, we can ignore fitness.
        with_indices.sort_by(|&(_, f1), &(_, f2)| f1.partial_cmp(f2).unwrap());

        // The rank of solution i now corresponds to the index in with_indices
        // of (i, fitness_i). But we want the original index to be the index,
        // and the rank to be used to generate the value.

        // Create a blank (not empty) vector, so that we can use random access
        // to sort by original index.
        let mut ranks = vec![0f64;with_indices.len()];
        for (rank_minus_one, &(index, _)) in with_indices.iter().enumerate() {
            ranks[index] = ((rank_minus_one + 1) as f64).powf(k);
        }
        ranks
    })
}