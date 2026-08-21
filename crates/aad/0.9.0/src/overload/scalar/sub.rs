#[macro_export(local_inner_macros)]
macro_rules! impl_scalar_sub_inner {
    ($scalar:ty, $one:expr, $zero:expr) => {
        #[inline]
        fn sub(self, rhs: $scalar) -> Self::Output {
            let value = &self.value - rhs;
            match &self.index {
                Some((i, tape)) => Variable {
                    index: {
                        let mut operations = tape.operations.borrow_mut();
                        let count = operations.len();
                        operations.push(OperationRecord([(*i, $one), (usize::MAX, $zero)]));
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
macro_rules! impl_scalar_sub {
    ($scalar:ty) => {
        impl<'a> Sub<$scalar> for &Variable<'a, $scalar>
        where
            for<'b> &'b $scalar: Sub<$scalar, Output = $scalar>,
        {
            type Output = Variable<'a, $scalar>;
            impl_scalar_sub_inner!($scalar, <$scalar>::one(), <$scalar>::zero());
        }

        impl<'a, 'b> Sub<$scalar> for &Variable<'a, Variable<'b, $scalar>>
        where
            for<'c> &'c $scalar: Sub<$scalar, Output = $scalar>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;
            impl_scalar_sub_inner!($scalar, Variable::one(), Variable::zero());
        }

        impl<'a> Sub<&Variable<'a, $scalar>> for $scalar
        where
            for<'b> &'b $scalar: Sub<$scalar, Output = $scalar>,
            for<'b> &'b Variable<'a, $scalar>: Neg<Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, $scalar>;

            #[inline]
            fn sub(self, rhs: &Variable<'a, $scalar>) -> Self::Output {
                -(rhs - self)
            }
        }

        impl<'a, 'b> Sub<&Variable<'a, Variable<'b, $scalar>>> for $scalar
        where
            for<'c> &'c Variable<'a, $scalar>: Sub<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;

            #[inline]
            fn sub(self, rhs: &Variable<'a, Variable<'b, $scalar>>) -> Self::Output {
                rhs - self
            }
        }

        impl<'a> Sub<&Variable<'a, $scalar>> for &$scalar
        where
            for<'b> &'b $scalar: Sub<$scalar, Output = $scalar>,
            for<'b> &'b Variable<'a, $scalar>: Neg<Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, $scalar>;

            #[inline]
            fn sub(self, rhs: &Variable<'a, $scalar>) -> Self::Output {
                -(rhs - *self)
            }
        }

        impl<'a, 'b> Sub<&Variable<'a, Variable<'b, $scalar>>> for &$scalar
        where
            for<'c> &'c Variable<'a, $scalar>: Sub<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;

            #[inline]
            fn sub(self, rhs: &Variable<'a, Variable<'b, $scalar>>) -> Self::Output {
                rhs - *self
            }
        }
    };
}
