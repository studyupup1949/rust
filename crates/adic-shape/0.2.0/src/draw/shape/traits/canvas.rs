use adic::traits::{CanTruncate, HasApproximateDigits};
use crate::error::AdicShapeResult;


/// An `AdicCanvas` can draw adic integers, numbers, and spaces, creating `Shapes`
pub trait AdicCanvas {

    /// Type of drawable that can then be made into svg or leptos components
    type Shape: Sized;

    /// Draw a single integer in the canvas
    fn draw_integer<'a, A>(
        &self,
        adic_integer: &'a A,
    ) -> AdicShapeResult<Self::Shape>
    where Self: sealed::DrawSingleInteger,
    A: Clone + HasApproximateDigits<DigitIndex = usize> + 'a {
        self._draw_integer(adic_integer)
    }

    /// Draw multiple integers in the canvas
    fn draw_integers<'a, A>(
        &self,
        adic_integers: impl IntoIterator<Item=&'a A>,
    ) -> AdicShapeResult<Self::Shape>
    where Self: sealed::DrawIntegers,
    A: Clone + HasApproximateDigits<DigitIndex = usize> + 'a {
        self._draw_integers(adic_integers)
    }

    /// Draw a single number in the canvas
    fn draw_number<'a, A>(
        &self,
        adic_number: &'a A,
    ) -> AdicShapeResult<Self::Shape>
    where Self: sealed::DrawSingleNumber,
    A: Clone + HasApproximateDigits<DigitIndex = isize> + CanTruncate + 'a,
    A::Quotient: Clone + HasApproximateDigits<DigitIndex = usize> {
        self._draw_number(adic_number)
    }

    /// Draw multiple numbers in the canvas
    fn draw_numbers<'a, A>(
        &self,
        adic_numbers: impl IntoIterator<Item=&'a A>,
    ) -> AdicShapeResult<Self::Shape>
    where Self: sealed::DrawNumbers,
    A: Clone + HasApproximateDigits<DigitIndex = isize> + CanTruncate + 'a {
        self._draw_numbers(adic_numbers)
    }

    /// Draw full adic integer space, `ZZ_p`, in the canvas
    fn draw_full(&self) -> AdicShapeResult<Self::Shape>
    where Self: sealed::DrawFullSpace {
        self._draw_full()
    }

}


pub (crate) mod sealed {

    use super::{AdicCanvas, AdicShapeResult, CanTruncate, HasApproximateDigits};

    /// Canvas can draw a single adic integer as a `Shape`
    pub trait DrawSingleInteger: AdicCanvas {

        /// Draw a single integer in the canvas
        fn _draw_integer(
            &self,
            adic_integer: &(impl Clone + HasApproximateDigits<DigitIndex = usize>),
        ) -> AdicShapeResult<Self::Shape>;

    }

    /// Canvas can draw multiple adic integers as a `Shape`
    pub trait DrawIntegers: AdicCanvas {

        /// Draw multiple integers in the canvas
        fn _draw_integers<'a, A>(
            &self,
            adic_integers: impl IntoIterator<Item=&'a A>,
        ) -> AdicShapeResult<Self::Shape>
        where A: Clone + HasApproximateDigits<DigitIndex = usize> + 'a;

    }

    /// Canvas can draw a single adic number as a `Shape`
    pub trait DrawSingleNumber: AdicCanvas {

        /// Draw a single number in the canvas
        fn _draw_number(
            &self,
            adic_number: &(impl Clone + HasApproximateDigits<DigitIndex = isize> + CanTruncate),
        ) -> AdicShapeResult<Self::Shape>;

    }

    /// Canvas can draw multiple adic integers as a `Shape`
    pub trait DrawNumbers: AdicCanvas {

        /// Draw multiple integers in the canvas
        fn _draw_numbers<'a, A>(
            &self,
            adic_numbers: impl IntoIterator<Item=&'a A>,
        ) -> AdicShapeResult<Self::Shape>
        where A: Clone + HasApproximateDigits<DigitIndex = isize> + CanTruncate + 'a;

    }

    /// Canvas can draw full adic space as a `Shape`
    pub trait DrawFullSpace: AdicCanvas {
        /// Draw full adic space in the canvas
        fn _draw_full(&self) -> AdicShapeResult<Self::Shape>;
    }

}
