#[macro_export(local_inner_macros)]
macro_rules! impl_scalar_add_inner {
    ($scalar:ty, $one:expr, $zero:expr) => {
        #[inline]
        fn add(self, rhs: $scalar) -> Self::Output {
            let value = &self.value + rhs;
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
macro_rules! impl_scalar_add {
    ($scalar:ty) => {
        impl<'a> Add<$scalar> for &Variable<'a, $scalar>
        where
            for<'b> &'b $scalar: Add<$scalar, Output = $scalar>,
        {
            type Output = Variable<'a, $scalar>;
            impl_scalar_add_inner!($scalar, <$scalar>::one(), <$scalar>::zero());
        }

        impl<'a, 'b> Add<$scalar> for &Variable<'a, Variable<'b, $scalar>>
        where
            for<'c> &'c $scalar: Add<$scalar, Output = $scalar>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;
            impl_scalar_add_inner!($scalar, Variable::one(), Variable::zero());
        }

        impl<'a> Add<&Variable<'a, $scalar>> for $scalar
        where
            for<'b> &'b Variable<'a, $scalar>: Add<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, $scalar>;

            #[inline]
            fn add(self, rhs: &Variable<'a, $scalar>) -> Self::Output {
                rhs + self
            }
        }

        impl<'a, 'b> Add<&Variable<'a, Variable<'b, $scalar>>> for $scalar
        where
            for<'c> &'c Variable<'a, $scalar>: Add<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;

            #[inline]
            fn add(self, rhs: &Variable<'a, Variable<'b, $scalar>>) -> Self::Output {
                rhs + self
            }
        }

        impl<'a> Add<&Variable<'a, $scalar>> for &$scalar
        where
            for<'b> &'b Variable<'a, $scalar>: Add<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, $scalar>;

            #[inline]
            fn add(self, rhs: &Variable<'a, $scalar>) -> Self::Output {
                rhs + *self
            }
        }

        impl<'a, 'b> Add<&Variable<'a, Variable<'b, $scalar>>> for &$scalar
        where
            for<'c> &'c Variable<'a, $scalar>: Add<$scalar, Output = Variable<'a, $scalar>>,
        {
            type Output = Variable<'a, Variable<'b, $scalar>>;

            #[inline]
            fn add(self, rhs: &Variable<'a, Variable<'b, $scalar>>) -> Self::Output {
                rhs + *self
            }
        }
    };
}
