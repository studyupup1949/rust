pub mod float_like;
pub mod gradients;
pub(crate) mod operation_record;
mod overload;
pub mod tape;
pub mod variable;

#[cfg(feature = "derive")]
pub use aad_derive::autodiff;
pub use float_like::FloatLike;
pub use tape::Tape;
pub use variable::Variable;

#[cfg(test)]
mod tests {
    use crate::Tape;

    #[test]
    fn test_add() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0);
        let y = tape.create_variable(3.0);
        let z = x + y;

        assert_eq!(z.value(), 5.0);

        let grads = z.compute_gradients().unwrap();
        assert_eq!(grads.get_gradients(&[x, y]).unwrap(), [1.0, 1.0]);
    }

    #[test]
    fn test_add_scalar() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0_f64);
        let z = x + 5.0;

        assert_eq!(z.value(), 7.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
    }

    #[test]
    fn test_neg() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0);
        let z = -x;

        assert_eq!(z.value(), -2.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), -1.0);
    }

    #[test]
    fn test_sub() {
        let tape = Tape::default();
        let x = tape.create_variable(5.0_f64);
        let y = tape.create_variable(3.0);
        let z = x - y;

        assert_eq!(z.value(), 2.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
        assert_eq!(grads.get_gradient(&y).unwrap(), -1.0);
    }

    #[test]
    fn test_sub_scalar() {
        let tape = Tape::default();
        let x = tape.create_variable(7.0_f64);
        let z = x - 4.0;

        assert_eq!(z.value(), 3.0);

        let grads = z.compute_gradients().unwrap();

        // 勾配計算: z = x - 4.0 => dz/dx = 1.0
        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
    }

    #[test]
    fn test_sub_from_scalar() {
        let tape = Tape::default();
        let x = tape.create_variable(7.0_f64);
        let z = 4.0 - x;

        assert_eq!(z.value(), -3.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), -1.0);
    }

    #[test]
    fn test_mul() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0);
        let y = tape.create_variable(3.0);
        let z = x * y;

        assert_eq!(z.value(), 6.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 3.0);
        assert_eq!(grads.get_gradient(&y).unwrap(), 2.0);
    }

    #[test]
    fn test_mul_scalar() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0_f64);
        let z = x * 5.0;

        assert_eq!(z.value(), 10.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 5.0);
    }

    #[test]
    fn test_div() {
        let tape = Tape::default();
        let x = tape.create_variable(6.0);
        let y = tape.create_variable(3.0);
        let z = x / y;

        assert_eq!(z.value(), 2.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0 / y.value());
        assert_eq!(
            grads.get_gradient(&y).unwrap(),
            -x.value() / (y.value() * y.value())
        );
    }

    #[test]
    fn test_div_scalar() {
        let tape = Tape::default();
        let x = tape.create_variable(8.0);
        let z = x / 4.0_f64;

        assert_eq!(z.value(), 2.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0 / 4.0);
    }

    #[test]
    fn test_div_scalar_reverse() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0_f64);
        let z = 10.0 / x;

        assert_eq!(z.value(), 5.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(
            grads.get_gradient(&x).unwrap(),
            -10.0 / (x.value() * x.value())
        );
    }

    #[test]
    fn test_linear_scalar() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0);
        let z = 2.0_f64 * x + 5.0_f64;

        assert_eq!(z.value(), 9.0);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 2.0);
    }

    #[test]
    fn test_sin() {
        let tape = Tape::default();
        let x = tape.create_variable(std::f64::consts::PI / 2.0);
        let z = x.sin();

        assert!((z.value() - 1.0).abs() < 1e-6);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap()).abs() < 1e-6);
    }

    #[test]
    fn test_recip() {
        let tape = Tape::default();
        let x = tape.create_variable(4.0);
        let z = x.recip();

        assert_eq!(z.value(), 0.25);

        let grads = z.compute_gradients().unwrap();

        assert_eq!(
            grads.get_gradient(&x).unwrap(),
            -1.0 / (x.value() * x.value())
        );
    }

    #[test]
    fn test_combined_operations() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0);
        let y = tape.create_variable(3.0);
        let z = (x + y) * y.sin();

        let expected_value = (2.0_f64 + 3.0_f64) * y.sin();
        assert!((z.value() - expected_value.value()).abs() < 1e-6);

        let grads = z.compute_gradients().unwrap();

        let grad_x = grads.get_gradient(&x).unwrap();
        let grad_y = grads.get_gradient(&y).unwrap();

        let expected_grad_x = y.sin().value();
        let expected_grad_y = (x.value() + y.value()) * y.value().cos() + y.value().sin();

        assert!((grad_x - expected_grad_x).abs() < 1e-6);
        assert!((grad_y - expected_grad_y).abs() < 1e-6);
    }

    #[test]
    fn test_combined_operations2() {
        // Test case from `Modern Computational Finance: AAD and Parallel Simulations` by Antoine Savine
        const MULTIPLIER: f64 = 5.0;

        let tape = Tape::default();

        let x0 = tape.create_variable(1.0);
        let x1 = tape.create_variable(2.0);
        let x2 = tape.create_variable(3.0);
        let x3 = tape.create_variable(4.0);
        let x4 = tape.create_variable(5.0);

        let y1 = x2 * ((MULTIPLIER * x0) + x1);
        let y2 = y1.ln();
        let y = (y1 + (x3 * y2)) * (y1 + y2);

        let grads = y.compute_gradients().unwrap();

        const EXPECTED_GRADIENTS: [f64; 5] = [
            950.7364539019619,
            190.14729078039238,
            443.6770118209156,
            73.20408806599326,
            0.0,
        ];

        assert_eq!(
            grads.get_gradients(&[x0, x1, x2, x3, x4]).unwrap(),
            EXPECTED_GRADIENTS
        );
    }

    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_powf() {
        let tape = Tape::default();
        let x = tape.create_variable(2.0);
        let z = x.powf(3.0.into());
        assert!((z.value() - 8.0_f64).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - 12.0).abs() < EPSILON);
    }

    #[test]
    fn test_exp() {
        let tape = Tape::default();
        let x = tape.create_variable(1.0);
        let z = x.exp();

        assert!((z.value() - f64::exp(1.0)).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - z.value()).abs() < EPSILON);
    }

    #[test]
    fn test_sqrt() {
        let tape = Tape::default();
        let x = tape.create_variable(4.0);
        let z = x.sqrt();

        assert!((z.value() - 2.0_f64).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - 0.25).abs() < EPSILON);
    }

    #[test]
    fn test_cos() {
        let tape = Tape::default();
        let x = tape.create_variable(0.0);
        let z = x.cos();

        assert!((z.value() - 1.0_f64).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_tan() {
        let tape = Tape::default();
        let x = tape.create_variable(0.0);
        let z = x.tan();

        assert!((z.value() - 0.0_f64).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_sinh() {
        let tape = Tape::default();
        let x = tape.create_variable(0.0);
        let z = x.sinh();

        assert!((z.value() - 0.0_f64).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_cosh() {
        let tape = Tape::default();
        let x = tape.create_variable(0.0);
        let z = x.cosh();

        assert!((z.value() - 1.0_f64).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_tanh() {
        let tape = Tape::default();
        let x = tape.create_variable(0.0);
        let z = x.tanh();

        assert!((z.value() - 0.0_f64).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_combined() {
        let tape = Tape::default();

        let x = tape.create_variable(1.0);
        let y = tape.create_variable(2.0);

        let z = (x + y).powf(3.0.into()) * x.exp();

        let expected = (1.0_f64 + 2.0_f64).powf(3.0) * f64::exp(1.0);

        assert!((z.value() - expected).abs() < EPSILON);

        let grads = z.compute_gradients().unwrap();

        let expected_grad_x = 3.0 * (x.value() + y.value()).powf(2.0) * x.exp().value() + z.value();

        assert!((grads.get_gradient(&x).unwrap() - expected_grad_x).abs() < EPSILON);
    }

    #[test]
    fn test_add_assign_scalar() {
        let tape = Tape::default();

        let mut x = tape.create_variable(8.0);

        x += 4.0;

        assert_eq!(x.value(), 12.0);

        let grads = x.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
    }

    #[test]
    fn test_add_assign_var() {
        let tape = Tape::default();

        let mut x = tape.create_variable(8.0);
        let y = tape.create_variable(3.0);

        x += y;
        x += y;

        assert_eq!(x.value(), 14.0);

        let grads = x.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
        assert_eq!(grads.get_gradient(&y).unwrap(), 2.0);
    }

    #[test]
    fn test_sub_assign_scalar() {
        let tape = Tape::default();

        let mut x = tape.create_variable(10.0);

        x -= 4.0;

        assert_eq!(x.value(), 6.0);

        let grads = x.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
    }

    #[test]
    fn test_sub_assign_var() {
        let tape = Tape::default();

        let mut x = tape.create_variable(7.0);
        let y = tape.create_variable(2.0);

        x -= y;
        x -= y;

        assert_eq!(x.value(), 3.0);

        let grads = x.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
        assert_eq!(grads.get_gradient(&y).unwrap(), -2.0);
    }

    #[test]
    fn test_mul_assign_scalar() {
        let tape = Tape::default();

        let mut x = tape.create_variable(3.0);

        x *= 2.0;

        assert_eq!(x.value(), 6.0);

        let grads = x.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
    }

    #[test]
    fn test_mul_assign_var() {
        let tape = Tape::default();

        let mut x = tape.create_variable(3.0);
        let y = tape.create_variable(2.0);

        x *= y;
        x *= y;

        assert_eq!(x.value(), 12.0);

        let grads = x.compute_gradients().unwrap();

        assert_eq!(grads.get_gradient(&x).unwrap(), 1.0);
        assert_eq!(grads.get_gradient(&y).unwrap(), 2.0 * x.value() / y.value());
    }

    #[test]
    fn test_div_assign_scalar() {
        let tape = Tape::default();

        let mut x = tape.create_variable(8.0_f64);

        x /= 4.0;

        assert_eq!(x.value(), 2.0);

        let grads = x.compute_gradients().unwrap();
        assert!((grads.get_gradient(&x).unwrap() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_div_assign_var() {
        let tape = Tape::default();

        let mut x = tape.create_variable(10.0_f64);
        let y = tape.create_variable(2.0);

        x /= y;
        x /= y;

        assert_eq!(x.value(), 2.5);

        let grads = x.compute_gradients().unwrap();

        let expected_dx = 1.0;
        let expected_dy = -2.0 * 10.0 / (y.value().powi(3));

        assert!((grads.get_gradient(&x).unwrap() - expected_dx).abs() < EPSILON);
        assert!((grads.get_gradient(&y).unwrap() - expected_dy).abs() < EPSILON);
    }

    #[test]
    fn test_i8() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_i8, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_i16() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_i16, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_i32() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_i32, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_i64() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_i64, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_i128() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_i128, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_isize() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_isize, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_u8() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_u8, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_u16() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_u16, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_u32() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_u32, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_u64() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_u64, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_u128() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_u128, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }

    #[test]
    fn test_usize() {
        let tape = Tape::new();
        let [x, y] = tape.create_variables(&[2_usize, 3]);

        let z = 2 * x + y;
        let grads = z.compute_gradients().unwrap();
        let res = grads.get_gradients(&[x, y]).unwrap();
        assert_eq!(res, [2, 1]);
    }
}
