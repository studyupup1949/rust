use itertools::Itertools;
use num::traits::Pow;
use crate::traits::{AdicPrimitive, HasApproximateDigits, HasDigits};
use super::{MAdic, PowAdic};



impl<A> std::ops::Add for PowAdic<A>
where A: AdicPrimitive + std::ops::Add<Output=A> {
    type Output = PowAdic<A>;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.power(), rhs.power(), "Only adic powers with same power can be added");
        let power = self.power();
        Self::new(self.adic + rhs.adic, power)
    }
}

impl<A> std::ops::AddAssign for PowAdic<A>
where A: AdicPrimitive + std::ops::AddAssign {
    fn add_assign(&mut self, rhs: Self) {
        assert_eq!(self.power(), rhs.power(), "Only adic powers with same power can be added");
        self.adic += rhs.adic;
    }
}


impl<A> std::ops::Neg for PowAdic<A>
where A: AdicPrimitive + std::ops::Neg<Output=A> {
    type Output = PowAdic<A>;
    fn neg(self) -> Self::Output {
        let power = self.power();
        Self::new(-self.adic, power)
    }
}


impl<A> std::ops::Sub for PowAdic<A>
where A: AdicPrimitive + std::ops::Neg<Output=A> {
    type Output = PowAdic<A>;
    fn sub(self, rhs: PowAdic<A>) -> Self::Output {
        self + (-rhs)
    }
}

impl<A> std::ops::SubAssign for PowAdic<A>
where A: AdicPrimitive + std::ops::SubAssign {
    fn sub_assign(&mut self, rhs: Self) {
        assert_eq!(self.power(), rhs.power(), "Only adic powers with same power can be added");
        self.adic -= rhs.adic;
    }
}


impl<A> std::ops::Mul for PowAdic<A>
where A: AdicPrimitive + std::ops::Mul<Output=A> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.power(), rhs.power(), "Only adic powers with same power can be multiplied");
        let power = self.power();
        Self::new(self.adic * rhs.adic, power)
    }
}

impl<A> std::ops::MulAssign for PowAdic<A>
where A: AdicPrimitive + std::ops::MulAssign {
    fn mul_assign(&mut self, rhs: Self) {
        assert_eq!(self.power(), rhs.power(), "Only adic powers with same power can be multiplied");
        self.adic *= rhs.adic;
    }
}

impl<A> Pow<u32> for PowAdic<A>
where A: AdicPrimitive + Pow<u32, Output = A> {
    type Output = PowAdic<A>;
    fn pow(self, power: u32) -> Self::Output {
        let self_power = self.power();
        Self::new(self.adic.pow(power), self_power)
    }
}


// Should probably impl Inv and Div too, giving a LazyDiv.
// But need to fix LazyDiv's use of usize/isize to include Ratio, first.


impl<A> std::ops::Add for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive + std::ops::Add<Output=A> {
    type Output = MAdic<A>;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.base(), rhs.base(), "Only adic composites with same base can be added");
        Self::new(
            self.pow_adics.into_values().sorted_by_key(PowAdic::p)
                .zip(rhs.pow_adics.into_values().sorted_by_key(PowAdic::p))
                .map(|(a, b)| a + b)
        )
    }
}

impl<A> std::ops::AddAssign for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive + std::ops::AddAssign {
    fn add_assign(&mut self, rhs: Self) {
        assert_eq!(self.base(), rhs.base(), "Only adic composites with same base can be added");
        self.pow_adics.values_mut().sorted_by_key(|a| a.p())
            .zip(rhs.pow_adics.into_values().sorted_by_key(PowAdic::p))
            .for_each(|(a, b)| { *a += b; });
    }
}


impl<A> std::ops::Neg for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive + std::ops::Neg<Output=A> {
    type Output = MAdic<A>;
    fn neg(self) -> Self::Output {
        Self::new(self.pow_adics.into_values().map(|a| -a))
    }
}


impl<A> std::ops::Sub for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive + std::ops::Neg<Output=A> {
    type Output = MAdic<A>;
    fn sub(self, rhs: MAdic<A>) -> Self::Output {
        self + (-rhs)
    }
}

impl<A> std::ops::SubAssign for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive + std::ops::SubAssign {
    fn sub_assign(&mut self, rhs: Self) {
        assert_eq!(self.base(), rhs.base(), "Only adic composites with same base can be added");
        self.pow_adics.values_mut().sorted_by_key(|a| a.p())
            .zip(rhs.pow_adics.into_values().sorted_by_key(PowAdic::p))
            .for_each(|(a, b)| { *a -= b; });
    }
}


impl<A> std::ops::Mul for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive + std::ops::Mul<Output=A> {
    type Output = MAdic<A>;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.base(), rhs.base(), "Only adic composites with same base can be multiplied");
        Self::new(
            self.pow_adics.into_values().sorted_by_key(PowAdic::p)
                .zip(rhs.pow_adics.into_values().sorted_by_key(PowAdic::p))
                .map(|(a, b)| a * b)
        )
    }
}

impl<A> std::ops::MulAssign for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive + std::ops::MulAssign {
    fn mul_assign(&mut self, rhs: Self) {
        assert_eq!(self.base(), rhs.base(), "Only adic composites with same base can be multiplied");
        self.pow_adics.values_mut().sorted_by_key(|a| a.p())
            .zip(rhs.pow_adics.into_values().sorted_by_key(PowAdic::p))
            .for_each(|(a, b)| { *a *= b; });
    }
}

impl<A> Pow<u32> for MAdic<A>
where A: HasApproximateDigits + AdicPrimitive + Pow<u32, Output = A> {
    type Output = MAdic<A>;
    fn pow(self, power: u32) -> Self::Output {
        Self::new(self.pow_adics.into_values().map(|a| a.pow(power)))
    }
}
