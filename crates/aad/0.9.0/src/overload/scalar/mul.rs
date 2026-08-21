#[macro_export(local_inner_macros)]
macro_rules! impl_scalar_mul_inner {
    ($scalar:ty, $zero:expr) => {
        #[inline]
        fn mul(self, rhs: $scalar) -> Self::Output {
            let value = &self.value * rhs;
            match &self.index {
                Some((i, tape)) => Variable {
                    index: {
                        let mut operations = tape.operations.borrow_mut();
                        let count = operations.len();
                        operations.push(OperationRecord([(*i, rhs), (usize::MAX, $zero)]));
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
macro_rules! impl_scalar_mul {
    ($scalar:ty) => {
        impl<'a> Mul<$scalar> for &Variable<'a, $scalar>
        where
            for<'b> &'b $scalar: Mul<$scalar, Output = $scalar>,
        {
            type Output = Variable<'a, $scalar>;
            impl_scalar_mul_inner!($scalar, <$scalar>::zero());
        }

        impl<'a, 'b> Mul<$scalar> for &Variable<'a, Variable<'b, $scalar>>
        where
            for<'c> &'c $scalar: Mul<$scalar, Output = $scalar>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;
            #[inline]
            fn mul(self, rhs: $scalar) -> Self::Output {
                let value = &self.value * rhs;
                match &self.index {
                    Some((i, tape)) => Variable {
                        index: {
                            let mut operations = tape.operations.borrow_mut();
                            let count = operations.len();
                            operations.push(OperationRecord([
                                (*i, Variable::constant(rhs)),
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

        impl<'a> Mul<&Variable<'a, $scalar>> for $scalar
        where
            for<'b> &'b Variable<'a, $scalar>: Mul<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, $scalar>;

            #[inline]
            fn mul(self, rhs: &Variable<'a, $scalar>) -> Self::Output {
                rhs * self
            }
        }

        impl<'a, 'b> Mul<&Variable<'a, Variable<'b, $scalar>>> for $scalar
        where
            for<'c> &'c Variable<'a, $scalar>: Mul<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;

            #[inline]
            fn mul(self, rhs: &Variable<'a, Variable<'b, $scalar>>) -> Self::Output {
                rhs * self
            }
        }

        impl<'a> Mul<&Variable<'a, $scalar>> for &$scalar
        where
            for<'b> &'b Variable<'a, $scalar>: Mul<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, $scalar>;

            #[inline]
            fn mul(self, rhs: &Variable<'a, $scalar>) -> Self::Output {
                rhs * *self
            }
        }

        impl<'a, 'b> Mul<&Variable<'a, Variable<'b, $scalar>>> for &$scalar
        where
            for<'c> &'c Variable<'a, $scalar>: Mul<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;

            #[inline]
            fn mul(self, rhs: &Variable<'a, Variable<'b, $scalar>>) -> Self::Output {
                rhs * *self
            }
        }
    };
}
