use itertools::Itertools;
use num::traits::Pow;
use crate::{AdicApproximate, AdicNumber, HasDigits};
use super::{AdicComposite, AdicPower};



impl<A> std::ops::Add for AdicPower<A>
where A: AdicNumber + std::ops::Add<Output=A> {
    type Output = AdicPower<A>;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.power(), rhs.power(), "Only adic powers with same power can be added");
        let power = self.power();
        Self::new(self.adic + rhs.adic, power)
    }
}


impl<A> std::ops::Neg for AdicPower<A>
where A: AdicNumber + std::ops::Neg<Output=A> {
    type Output = AdicPower<A>;
    fn neg(self) -> Self::Output {
        let power = self.power();
        Self::new(-self.adic, power)
    }
}


impl<A> std::ops::Sub for AdicPower<A>
where A: AdicNumber + std::ops::Neg<Output=A> {
    type Output = AdicPower<A>;
    fn sub(self, rhs: AdicPower<A>) -> Self::Output {
        self + (-rhs)
    }
}

impl<A> std::ops::Mul for AdicPower<A>
where A: AdicNumber + std::ops::Mul<Output=A> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.power(), rhs.power(), "Only adic powers with same power can be multiplied");
        let power = self.power();
        Self::new(self.adic * rhs.adic, power)
    }
}

impl<A> std::ops::Mul<u32> for AdicPower<A>
where A: AdicNumber + std::ops::Mul<u32, Output=A> {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        let power = self.power();
        Self::new(self.adic * rhs, power)
    }
}

impl<A> std::ops::Mul<AdicPower<A>> for u32
where A: AdicNumber, u32: std::ops::Mul<A, Output=A> {
    type Output = AdicPower<A>;
    fn mul(self, rhs: AdicPower<A>) -> Self::Output {
        let power = rhs.power();
        AdicPower::new(self * rhs.adic, power)
    }
}


impl<A> Pow<u32> for AdicPower<A>
where A: AdicNumber + Pow<u32, Output = A> {
    type Output = AdicPower<A>;
    fn pow(self, power: u32) -> Self::Output {
        let self_power = self.power();
        Self::new(self.adic.pow(power), self_power)
    }
}


// Should probably impl Inv and Div too, giving a LazyDiv.
// But need to fix LazyDiv's use of usize/isize to include Ratio, first.


impl<A> std::ops::Add for AdicComposite<A>
where A: AdicApproximate + AdicNumber + std::ops::Add<Output=A> {
    type Output = AdicComposite<A>;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.base(), rhs.base(), "Only adic composites with same base can be added");
        Self::new(
            self.p_adics.into_values().sorted_by_key(AdicPower::p)
                .zip(rhs.p_adics.into_values().sorted_by_key(AdicPower::p))
                .map(|(a, b)| a + b)
        )
    }
}


impl<A> std::ops::Neg for AdicComposite<A>
where A: AdicApproximate + AdicNumber + std::ops::Neg<Output=A> {
    type Output = AdicComposite<A>;
    fn neg(self) -> Self::Output {
        Self::new(self.p_adics.into_values().map(|a| -a))
    }
}


impl<A> std::ops::Sub for AdicComposite<A>
where A: AdicApproximate + AdicNumber + std::ops::Neg<Output=A> {
    type Output = AdicComposite<A>;
    fn sub(self, rhs: AdicComposite<A>) -> Self::Output {
        self + (-rhs)
    }
}


impl<A> std::ops::Mul for AdicComposite<A>
where A: AdicApproximate + AdicNumber + std::ops::Mul<Output=A> {
    type Output = AdicComposite<A>;
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.base(), rhs.base(), "Only adic composites with same base can be multiplied");
        Self::new(
            self.p_adics.into_values().sorted_by_key(AdicPower::p)
                .zip(rhs.p_adics.into_values().sorted_by_key(AdicPower::p))
                .map(|(a, b)| a * b)
        )
    }
}

impl<A> std::ops::Mul<u32> for AdicComposite<A>
where A: AdicApproximate + AdicNumber + std::ops::Mul<u32, Output=A> {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self::new(self.p_adics.into_values().map(|a| a * rhs))
    }
}

impl<A> std::ops::Mul<AdicComposite<A>> for u32
where A: AdicApproximate + AdicNumber, u32: std::ops::Mul<A, Output=A> {
    type Output = AdicComposite<A>;
    fn mul(self, rhs: AdicComposite<A>) -> Self::Output {
        AdicComposite::new(rhs.p_adics.into_values().map(|a| std::ops::Mul::<AdicPower<A>>::mul(self, a)))
    }
}


impl<A> Pow<u32> for AdicComposite<A>
where A: AdicApproximate + AdicNumber + Pow<u32, Output = A> {
    type Output = AdicComposite<A>;
    fn pow(self, power: u32) -> Self::Output {
        Self::new(self.p_adics.into_values().map(|a| a.pow(power)))
    }
}
