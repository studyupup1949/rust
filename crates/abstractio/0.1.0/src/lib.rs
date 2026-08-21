#![deny(missing_docs)]

//! # Abstractio
//! Abstract IO dimensionality analysis for physics using theory of Avatar Extensions
//!
//! Brought to you by the [AdvancedResearch](https://advancedresearch.github.io/) community.
//!
//! [Join us on Discord!](https://discord.gg/JkrhJJRBR2)
//!
//! ```rust
//! use abstractio::*;
//!
//! fn main() {
//!     assert_eq!(density().dim(), [1, 2]);
//!     assert_eq!(measure_force().dim(), [3, 7]);
//!
//!     assert_eq!(format!("{:?}", density().to_abstract()), "Bin((Variable, Variable))".to_string());
//!     assert_eq!(density().to_abstract().dim(), [1, 2]);
//! }
//! ```
//!
//! IO dimensionality can be used to determine freedom of degrees in a physical system
//! and the work required to measure it.
//!
//! Abstract IO dimensionality analysis is a structure that can be projected down from the algebraic
//! description of the physical system and used to calculate the IO dimensionality without
//! loss of information.
//!
//! ### Motivation
//!
//! The theory of [Avatar Extensions](https://advancedresearch.github.io/avatar-extensions/summary.html)
//! predicts that there is a level of abstraction where the concrete binary operators do not matter
//! and where unary operators contract topologically.
//! In particular, this kind of analysis is important for the semantics of [Avatar Graphs](https://advancedresearch.github.io/avatar-extensions/summary.html#avatar-graphs).
//!
//! This library shows that this level of abstraction is possible,
//! using combinatorial properties of "ways to read" and "ways to write".
//!
//! An algebraic expression describing a physical system or measurement
//! is analyzed and an IO dimension vector is calculated.
//! The abstraction level made possible here is shown by projecting the algebraic
//! expression to an "Abstract IO" data structure.
//! From this abstract structure, it is possible to compute the IO dimension vector without
//! loss of information.
//!
//! Explained in [Path Semantical](https://github.com/advancedresearch/path_semantics) notation:
//!
//! ```text
//! ∴ f[dim_io] <=> f[to_abstract][dim_abstract_io]
//!
//! ∵ dim_io <=> dim_abstract_io . to_abstract
//! ```
//!
//! This is a tautology in Path Semantics.
//! However, it is not given that `f[to_abstract]` has a solution,
//! since Path Semantics has an imaginary inverse operator.
//!
//! The abstraction is possible if and only if `f[to_abstract]` has a solution.
//! This property is demonstrated in this library.
//!
//! ### Design
//!
//! An IO dimension vector is a pair of natural numbers that counts
//! the number of "ways to read" and "ways to write" in a physical system.
//!
//! `[<ways to read>, <ways to write>]`
//!
//! For example, a constant has IO dimensions `[1, 0]`.
//! The first number is 1 since there is one way to read the value of a constant.
//! The second number is 0 since it is not possible to change a constant.
//!
//! Another example: A variable has IO dimensions `[1, 1]`.
//! The first number is 1 since there is one way to read the value of a variable.
//! The second number is 1 since there is one way to write a new value to a variable.
//!
//! A more complex example is `density := mass / volume`.
//! This system has IO dimensions `[1, 2]`.
//! The first dimension is 1 since there is one way to read the value of density.
//! The second dimension is 2 since there are two ways to write a new value to a density,
//! one way where volume is kept constant and one way where mass is kept constant.

use Io::*;

/// Represents abstract IO physical expression.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AbstractIo {
    /// Constant.
    Constant,
    /// Variable.
    Variable,
    /// Binary operator with symbolic distinction.
    Bin(Box<(AbstractIo, AbstractIo)>),
    /// Vector.
    Vector(u64, Box<AbstractIo>),
    /// Tuple.
    Tup(Vec<AbstractIo>),
}

/// Represents IO physical expression.
#[derive(Clone, PartialEq)]
pub enum Io {
    // Constants.
    /// A 0-avatar constant used in sums and products.
    ///
    /// This can be thought of as a shared identity element
    /// of addition and multiplication that takes on that identity
    /// when being used as an argument.
    AvatarCore,
    /// Avogadros constant.
    Avogadro,
    /// Boltzmann constant.
    Boltzmann,
    /// Numeric constant.
    Const(f64),
    /// Gravity constant.
    Gravity,
    /// Luminous efficacy of 540 THz radiation.
    LuminousEfficacy,
    /// Planck constant.
    Planck,
    /// Speed of light constant.
    SpeedOfLight,
    // Variables.
    /// Acceleration variable.
    Acceleration,
    /// Amount of substance variable.
    AmountOfSubstance,
    /// Charge variable.
    Charge,
    /// Electric current variable.
    ElectricCurrent,
    /// Length variable.
    Length,
    /// Luminous intensity variable.
    LuminousIntensity,
    /// Mass variable.
    Mass,
    /// Speed variable.
    Speed,
    /// Thermodynamic temperature variable.
    ThermodynamicTemperature,
    /// Time variable.
    Time,
    /// Volume variable.
    Volume,
    // Operators.
    /// Negation unary operator.
    Neg(Box<Io>),
    /// Square root unary operator.
    Sqrt(Box<Io>),
    /// Addition binary operator.
    Add(Box<(Io, Io)>),
    /// Subtraction binary operator.
    Sub(Box<(Io, Io)>),
    /// Multiply binary operator.
    Mul(Box<(Io, Io)>),
    /// Division binary operator.
    Div(Box<(Io, Io)>),
    /// Vector.
    Vector(u64, Box<Io>),
    /// Tuple.
    Tup(Vec<Io>),
    /// Vector component.
    Comp(u64, Box<Io>),
}

impl AbstractIo {
    /// Calculate the abstract IO dimension.
    ///
    /// This is a vector counting `[<ways to read>, <ways to write>]`.
    pub fn dim(&self) -> [u64; 2] {
        use AbstractIo::*;

        match self {
            Constant => [1, 0],
            Variable => [1, 1],
            Bin(ab) => {
                let a_dim = ab.0.dim();
                let b_dim = ab.1.dim();
                [a_dim[0] * b_dim[0], a_dim[1] + b_dim[1]]
            }
            Vector(n, a) => {
                let a_dim = a.dim();
                [n * a_dim[0], n * a_dim[1]]
            }
            Tup(items) => {
                let mut sum = [0, 0];
                for it in items {
                    let dim = it.dim();
                    sum = [sum[0] + dim[0], sum[1] + dim[1]];
                }
                sum
            }
        }
    }
}

impl Io {
    /// Returns `true` when self is being same as other.
    pub fn same(&self, other: &Io) -> bool {
        self == other
    }

    /// Takes the norm if vector.
    pub fn norm(&self) -> Option<Io> {
        if let Vector(n, a) = self {Some(norm(*n, (**a).clone()))} else {None}
    }
}

impl Io {
    /// Converts to abstract IO structure.
    pub fn to_abstract(&self) -> AbstractIo {
        match self {
            AvatarCore |
            Avogadro |
            Boltzmann |
            Charge |
            Const(_) |
            Gravity |
            LuminousEfficacy |
            Planck |
            SpeedOfLight => AbstractIo::Constant,

            Acceleration |
            AmountOfSubstance |
            ElectricCurrent |
            Length |
            LuminousIntensity |
            Mass |
            Speed |
            ThermodynamicTemperature |
            Time |
            Volume => AbstractIo::Variable,

            Neg(a) |
            Sqrt(a) => a.to_abstract(),
            Add(ab) | Sub(ab) | Mul(ab) | Div(ab) => {
                if ab.0.same(&ab.1) {return ab.0.to_abstract()};

                AbstractIo::Bin(Box::new((ab.0.to_abstract(), ab.1.to_abstract())))
            }
            Vector(n, a) => AbstractIo::Vector(*n, Box::new(a.to_abstract())),
            Tup(items) => AbstractIo::Tup(items.iter().map(|n| n.to_abstract()).collect()),
            Comp(_, a) => a.to_abstract(),
        }
    }
}

impl Io {
    /// Gets the IO dimension.
    pub fn dim(&self) -> [u64; 2] {self.to_abstract().dim()}
}

/// Negation unary operator.
pub fn neg(a: Io) -> Io {Neg(Box::new(a))}
/// Square root unary operator.
pub fn sqrt(a: Io) -> Io {Sqrt(Box::new(a))}
/// Addition binary operator.
pub fn add(a: Io, b: Io) -> Io {Add(Box::new((a, b)))}
/// Subtraction binary operator.
pub fn sub(a: Io, b: Io) -> Io {Sub(Box::new((a, b)))}
/// Multiplication binary operator.
pub fn mul(a: Io, b: Io) -> Io {Mul(Box::new((a, b)))}
/// Division operator.
pub fn div(a: Io, b: Io) -> Io {Div(Box::new((a, b)))}
/// Square operator.
pub fn square(a: Io) -> Io {mul(a.clone(), a)}

/// A vector with 3 components.
pub fn vec3(a: Io) -> Io {Vector(3, Box::new(a))}
/// A tuple with 2 elements.
pub fn tup2(a: Io, b: Io) -> Io {Tup(vec![a, b])}
/// A tuple with 3 elements.
pub fn tup3(a: Io, b: Io, c: Io) -> Io {Tup(vec![a, b, c])}

/// Sum over some vector.
pub fn sum(n: u64, a: Io) -> Io {
    let mut res = AvatarCore;
    for i in 0..n {
        res = add(res, Comp(i, Box::new(a.clone())));
    }
    res
}

/// Product over some vector.
pub fn prod(n: u64, a: Io) -> Io {
    let mut res = AvatarCore;
    for i in 0..n {
        res = mul(res, Comp(i, Box::new(a.clone())));
    }
    res
}

/// Abstract binary operator.
pub fn bin(a: AbstractIo, b: AbstractIo) -> AbstractIo {
    AbstractIo::Bin(Box::new((a, b)))
}

/// Abstract fold (e.g. projected sums and products).
pub fn fold(n: u64, a: AbstractIo) -> AbstractIo {
    let mut res = AbstractIo::Constant;
    for _ in 0..n {
        res = bin(res, a.clone());
    }
    res
}

/// The norm squared of some vector.
pub fn norm_squared(n: u64, a: Io) -> Io {sum(n, square(a))}
/// The norm of some vector.
pub fn norm(n: u64, a: Io) -> Io {sqrt(norm_squared(n, a))}
/// The average over some vector.
pub fn avg(n: u64, a: Io) -> Io {div(sum(n, a), Const(n as f64))}

/// Speed of light constant.
pub fn c() -> Io {SpeedOfLight}
/// Density.
pub fn density() -> Io {mul(Mass, Volume)}
/// Rest mass energy.
pub fn rest_mass_energy() -> Io {mul(Mass, square(c()))}
/// Position.
pub fn position() -> Io {vec3(Length)}
/// Velocity.
pub fn velocity() -> Io {vec3(Speed)}
/// Acceleration vector.
pub fn acceleration_vector() -> Io {vec3(Acceleration)}
/// Force.
pub fn force() -> Io {mul(Mass, acceleration_vector())}
/// Energy.
pub fn energy() -> Io {mul(mul(Mass, Acceleration), Length)}
/// Gravity potential energy.
pub fn gravity_potential_energy() -> Io {mul(Gravity, mul(Mass, Length))}
/// Kinetic energy.
pub fn kinetic_energy() -> Io {mul(Const(0.5), mul(Mass, square(Speed)))}

/// Measure speed.
pub fn measure_speed() -> Io {div(Length, Time)}
/// Measure speed between lengths.
pub fn measure_speed_between_lengths() -> Io {div(sub(Length, Length), Time)}
/// Measure velocity.
pub fn measure_velocity() -> Io {vec3(measure_speed())}
/// Measure acceleration.
pub fn measure_acceleration() -> Io {div(Length, square(Time))}
/// Measure acceleration vector.
pub fn measure_acceleration_vector() -> Io {vec3(measure_acceleration())}
/// Measure force.
pub fn measure_force() -> Io {mul(Mass, measure_acceleration_vector())}
/// Measure energy.
pub fn measure_energy() -> Io {mul(mul(Mass, measure_acceleration()), Length)}
/// Measur gravity potential energy.
pub fn measure_gravity_potential_energy() -> Io {gravity_potential_energy()}
/// Measure kinetic energy.
pub fn measure_kinetic_energy() -> Io {kinetic_energy()}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_density() {
        let a = density();
        assert_eq!(a.dim(), [1, 2]);
    }

    #[test]
    fn test_square() {
        let a = square(Mass);
        assert_eq!(a.dim(), [1, 1]);
    }

    #[test]
    fn test_commute() {
        let a = mul(c(), mul(Mass, c()));
        let b = rest_mass_energy();
        assert_eq!(b.dim(), [1, 1]);
        assert_eq!(a.dim(), b.dim());
    }

    #[test]
    fn test_speed() {
        let a = measure_speed();
        assert_eq!(a.dim(), [1, 2]);

        let b = measure_speed_between_lengths();
        assert_eq!(b.dim(), [1, 2]);
        assert_eq!(a.dim(), b.dim());
    }

    #[test]
    fn test_velocity() {
        let a = velocity();
        assert_eq!(a.dim(), [3, 3]);

        let a = measure_velocity();
        assert_eq!(a.dim(), [3, 6]);
    }

    #[test]
    fn test_acceleration() {
        let a = measure_acceleration();
        assert_eq!(a.dim(), [1, 2]);

        let a = acceleration_vector();
        assert_eq!(a.dim(), [3, 3]);

        let a = measure_acceleration_vector();
        assert_eq!(a.dim(), [3, 6]);
    }

    #[test]
    fn test_force() {
        let a = force();
        assert_eq!(a.dim(), [3, 4]);

        let a = measure_force();
        assert_eq!(a.dim(), [3, 7]);
    }

    #[test]
    fn test_energy() {
        let a = energy();
        assert_eq!(a.dim(), [1, 3]);

        let a = measure_energy();
        assert_eq!(a.dim(), [1, 4]);
    }

    #[test]
    fn test_tup() {
        let a = tup2(measure_force(), measure_energy());
        assert_eq!(a.dim(), [4, 11]);
    }

    #[test]
    fn test_gravity_potential_energy() {
        let a = gravity_potential_energy();
        assert_eq!(a.dim(), [1, 2]);

        let a = measure_gravity_potential_energy();
        assert_eq!(a.dim(), [1, 2]);
    }

    #[test]
    fn test_kinetic_energy() {
        let a = kinetic_energy();
        assert_eq!(a.dim(), [1, 2]);

        let a = measure_kinetic_energy();
        assert_eq!(a.dim(), [1, 2]);
    }

    #[test]
    fn test_sub() {
        let a = sub(Length, Time);
        let b = add(Length, neg(Time));
        assert_eq!(a.dim(), b.dim());
    }

    #[test]
    fn test_norm() {
        let a = norm(3, measure_speed());
        assert_eq!(a.dim(), [1, 6]);

        let b = measure_velocity();
        assert_eq!(b.dim(), [3, 6]);
        assert!(a.dim() != b.dim());

        let c = measure_velocity().norm().unwrap();
        assert_eq!(c.dim(), [1, 6]);
        assert_eq!(a.dim(), c.dim());
    }

    #[test]
    fn test_fold() {
        let n = 10;
        let a = sum(n, Length);
        let b = fold(n, Length.to_abstract());
        assert_eq!(a.dim(), b.dim());

        let a = sum(n, measure_speed());
        let b = fold(n, measure_speed().to_abstract());
        assert_eq!(a.dim(), b.dim());
    }
}
