#[macro_export(local_inner_macros)]
macro_rules! impl_scalar_div_inner {
    ($scalar:ty, $zero:expr) => {
        #[inline]
        fn div(self, rhs: $scalar) -> Self::Output {
            let value = &self.value / rhs;
            match &self.index {
                Some((i, tape)) => Variable {
                    index: {
                        let mut operations = tape.operations.borrow_mut();
                        let count = operations.len();
                        operations.push(OperationRecord([(*i, rhs.recip()), (usize::MAX, $zero)]));
                        Some((count, tape))
                    },
                    value,
                },
                None => Variable { index: None, value },
            }
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! impl_scalar_div {
    ($scalar:ty) => {
        impl<'a> Div<$scalar> for &Variable<'a, $scalar>
        where
            for<'b> &'b $scalar: Div<$scalar, Output = $scalar>,
        {
            type Output = Variable<'a, $scalar>;
            impl_scalar_div_inner!($scalar, <$scalar>::zero());
        }

        impl<'a, 'b> Div<$scalar> for &Variable<'a, Variable<'b, $scalar>>
        where
            for<'c> &'c $scalar: Div<$scalar, Output = $scalar>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;
            #[inline]
            fn div(self, rhs: $scalar) -> Self::Output {
                let value = &self.value / rhs;
                match &self.index {
                    Some((i, tape)) => Variable {
                        index: {
                            let mut operations = tape.operations.borrow_mut();
                            let count = operations.len();
                            operations.push(OperationRecord([
                                (*i, Variable::constant(rhs.recip())),
                                (usize::MAX, Variable::zero()),
                            ]));
                            Some((count, tape))
                        },
                        value,
                    },
                    None => Variable { index: None, value },
                }
            }
        }

        impl<'a> Div<&Variable<'a, $scalar>> for $scalar
        where
            for<'b> &'b Variable<'a, $scalar>: Div<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, $scalar>;

            #[inline]
            fn div(self, rhs: &Variable<'a, $scalar>) -> Self::Output {
                &self / rhs
            }
        }

        impl<'a, 'b> Div<&Variable<'a, Variable<'b, $scalar>>> for $scalar
        where
            for<'c> &'c Variable<'a, Variable<'b, $scalar>>:
                Div<$scalar, Output = Variable<'a, Variable<'b, $scalar>>>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;

            #[inline]
            fn div(self, rhs: &Variable<'a, Variable<'b, $scalar>>) -> Self::Output {
                &self / rhs
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl<'a> Div<&Variable<'a, $scalar>> for &$scalar
        where
            Variable<'a, $scalar>:
                Inv<Output = Variable<'a, $scalar>> + Mul<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, $scalar>;

            #[inline]
            fn div(self, rhs: &Variable<'a, $scalar>) -> Self::Output {
                rhs.inv() * *self
            }
        }

        impl<'a, 'b> Div<&Variable<'a, Variable<'b, $scalar>>> for &$scalar
        where
            for<'c> &'c Variable<'a, Variable<'b, $scalar>>:
                Div<$scalar, Output = Variable<'a, Variable<'b, $scalar>>>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;

            #[inline]
            fn div(self, rhs: &Variable<'a, Variable<'b, $scalar>>) -> Self::Output {
                *self / rhs
            }
        }
    };
}
