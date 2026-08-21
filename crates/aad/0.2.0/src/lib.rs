pub(crate) mod grads;
pub(crate) mod operations;
pub mod tape;
pub(crate) mod var;

#[cfg(test)]
mod tests {
    use crate::tape::Tape;

    #[test]
    fn test_add() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let y = tape.var(3.0);
        let z = x + y;

        assert_eq!(z.value(), 5.0);

        let grads = z.backward();

        assert_eq!(grads.get(&[x, y]), [1.0, 1.0]);
    }

    #[test]
    fn test_add_scalar() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let z = x + 5.0;

        assert_eq!(z.value(), 7.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), 1.0);
    }

    #[test]
    fn test_neg() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let z = -x;

        assert_eq!(z.value(), -2.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), -1.0);
    }

    #[test]
    fn test_sub() {
        let tape = Tape::default();
        let x = tape.var(5.0);
        let y = tape.var(3.0);
        let z = x - y;

        assert_eq!(z.value(), 2.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), 1.0);
        assert_eq!(grads.get_one(&y), -1.0);
    }

    #[test]
    fn test_sub_scalar() {
        let tape = Tape::default();
        let x = tape.var(7.0);
        let z = x - 4.0;

        assert_eq!(z.value(), 3.0);

        let grads = z.backward();

        // 勾配計算: z = x - 4.0 => dz/dx = 1.0
        assert_eq!(grads.get_one(&x), 1.0);
    }

    #[test]
    fn test_sub_from_scalar() {
        let tape = Tape::default();
        let x = tape.var(7.0);
        let z = 4.0 - x;

        assert_eq!(z.value(), -3.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), -1.0);
    }

    #[test]
    fn test_mul() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let y = tape.var(3.0);
        let z = x * y;

        assert_eq!(z.value(), 6.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), 3.0);
        assert_eq!(grads.get_one(&y), 2.0);
    }

    #[test]
    fn test_mul_scalar() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let z = x * 5.0;

        assert_eq!(z.value(), 10.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), 5.0);
    }

    #[test]
    fn test_div() {
        let tape = Tape::default();
        let x = tape.var(6.0);
        let y = tape.var(3.0);
        let z = x / y;

        assert_eq!(z.value(), 2.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), 1.0 / y.value());
        assert_eq!(grads.get_one(&y), -x.value() / (y.value() * y.value()));
    }

    #[test]
    fn test_div_scalar() {
        let tape = Tape::default();
        let x = tape.var(8.0);
        let z = x / 4.0;

        assert_eq!(z.value(), 2.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), 1.0 / 4.0);
    }

    #[test]
    fn test_div_scalar_reverse() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let z = 10.0 / x;

        assert_eq!(z.value(), 5.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), -10.0 / (x.value() * x.value()));
    }

    #[test]
    fn test_linear_scalar() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let z = 2.0 * x + 5.0;

        assert_eq!(z.value(), 9.0);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), 2.0);
    }

    #[test]
    fn test_sin() {
        let tape = Tape::default();
        let x = tape.var(std::f64::consts::PI / 2.0);
        let z = x.sin();

        assert!((z.value() - 1.0).abs() < 1e-6);

        let grads = z.backward();
        assert!(grads.get_one(&x).abs() < 1e-6);
    }

    #[test]
    fn test_recip() {
        let tape = Tape::default();
        let x = tape.var(4.0);
        let z = x.recip();

        assert_eq!(z.value(), 0.25);

        let grads = z.backward();

        assert_eq!(grads.get_one(&x), -1.0 / (x.value() * x.value()));
    }

    #[test]
    fn test_combined_operations() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let y = tape.var(3.0);
        let z = (x + y) * y.sin();

        let expected_value = (2.0 + 3.0) * y.sin();
        assert!((z.value() - expected_value.value()).abs() < 1e-6);

        let grads = z.backward();

        let grad_x = grads.get_one(&x);
        let grad_y = grads.get_one(&y);

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

        let x0 = tape.var(1.0);
        let x1 = tape.var(2.0);
        let x2 = tape.var(3.0);
        let x3 = tape.var(4.0);
        let x4 = tape.var(5.0);

        let y1 = x2 * ((MULTIPLIER * x0) + x1);
        let y2 = y1.ln();
        let y = (y1 + (x3 * y2)) * (y1 + y2);

        let grads = y.backward();

        let expected_gradients = [
            950.7364539019619,  // grad_x0
            190.14729078039238, // grad_x1
            443.6770118209156,  // grad_x2
            73.20408806599326,  // grad_x3
            0.0,                // grad_x4
        ];

        assert_eq!(grads.get(&[x0, x1, x2, x3, x4]), expected_gradients);
    }

    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_powf() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let z = x.powf(3.0);
        assert!((z.value() - 8.0).abs() < EPSILON); // 2^3 = 8

        let grads = z.backward();
        assert!((grads.get_one(&x) - 12.0).abs() < EPSILON); // d(2^3)/dx = 3 * 2^(3-1) = 12
    }

    #[test]
    fn test_exp() {
        let tape = Tape::default();
        let x = tape.var(1.0);
        let z = x.exp();

        assert!((z.value() - f64::exp(1.0)).abs() < EPSILON); // e^1 = exp(1)

        let grads = z.backward();
        assert!((grads.get_one(&x) - z.value()).abs() < EPSILON); // d(e^x)/dx = e^x
    }

    #[test]
    fn test_sqrt() {
        let tape = Tape::default();
        let x = tape.var(4.0);
        let z = x.sqrt();

        assert!((z.value() - 2.0).abs() < EPSILON); // sqrt(4) = 2

        let grads = z.backward();
        assert!((grads.get_one(&x) - 0.25).abs() < EPSILON); // d(sqrt(4))/dx = 1 / (2 * sqrt(4)) = 0.25
    }

    #[test]
    fn test_cos() {
        let tape = Tape::default();
        let x = tape.var(0.0);
        let z = x.cos();

        assert!((z.value() - 1.0).abs() < EPSILON); // cos(0) = 1

        let grads = z.backward();
        assert!((grads.get_one(&x) - 0.0).abs() < EPSILON); // d(cos(0))/dx = -sin(0) = 0
    }

    #[test]
    fn test_tan() {
        let tape = Tape::default();
        let x = tape.var(0.0);
        let z = x.tan();

        assert!((z.value() - 0.0).abs() < EPSILON); // tan(0) = 0

        let grads = z.backward();
        assert!((grads.get_one(&x) - 1.0).abs() < EPSILON); // d(tan(0))/dx = 1 / cos^2(0) = 1
    }

    #[test]
    fn test_sinh() {
        let tape = Tape::default();
        let x = tape.var(0.0);
        let z = x.sinh();

        assert!((z.value() - 0.0).abs() < EPSILON); // sinh(0) = 0

        let grads = z.backward();
        assert!((grads.get_one(&x) - 1.0).abs() < EPSILON); // d(sinh(0))/dx = cosh(0) = 1
    }

    #[test]
    fn test_cosh() {
        let tape = Tape::default();
        let x = tape.var(0.0);
        let z = x.cosh();

        assert!((z.value() - 1.0).abs() < EPSILON); // cosh(0) = 1

        let grads = z.backward();
        assert!((grads.get_one(&x) - 0.0).abs() < EPSILON); // d(cosh(0))/dx = sinh(0) = 0
    }

    #[test]
    fn test_tanh() {
        let tape = Tape::default();
        let x = tape.var(0.0);
        let z = x.tanh();

        assert!((z.value() - 0.0).abs() < EPSILON); // tanh(0) = 0

        let grads = z.backward();
        assert!((grads.get_one(&x) - 1.0).abs() < EPSILON); // d(tanh(0))/dx = 1 - tanh(0)^2 = 1
    }

    #[test]
    fn test_combined() {
        let tape = Tape::default();

        let x = tape.var(1.0);
        let y = tape.var(2.0);

        // z = (x + y).powf(3) * x.exp()
        let z = (x + y).powf(3.0) * x.exp();

        let expected = (1.0_f64 + 2.0_f64).powf(3.0) * f64::exp(1.0);

        assert!((z.value() - expected).abs() < EPSILON);

        let grads = z.backward();

        // 手計算による勾配（例としてxの勾配のみ計算）
        let expected_grad_x = 3.0 * (x.value() + y.value()).powf(2.0) * x.exp().value() + z.value();

        assert!((grads.get_one(&x) - expected_grad_x).abs() < EPSILON);
    }
}
