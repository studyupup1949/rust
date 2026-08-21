pub mod core;

#[cfg(test)]
mod tests {
    use crate::core::tape::Tape;

    #[test]
    fn test_add() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let y = tape.var(3.0);
        let z = x + y;

        assert_eq!(z.value(), 5.0);

        z.backward();

        assert_eq!(x.grad(), 1.0);
        assert_eq!(y.grad(), 1.0);
    }

    #[test]
    fn test_mul() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let y = tape.var(3.0);
        let z = x * y;

        assert_eq!(z.value(), 6.0);

        z.backward();

        assert_eq!(x.grad(), 3.0);
        assert_eq!(y.grad(), 2.0);
    }

    #[test]
    fn test_sin() {
        let tape = Tape::default();
        let x = tape.var(std::f64::consts::PI / 2.0);
        let z = x.sin();

        assert!((z.value() - 1.0).abs() < 1e-6);

        z.backward();

        assert!((x.grad() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_combined_operations() {
        let tape = Tape::default();
        let x = tape.var(2.0);
        let y = tape.var(3.0);
        let z = (x + y) * y.sin();

        let expected_value = (2.0 + 3.0) * y.sin();
        assert!((z.value() - expected_value.value()).abs() < 1e-6);

        z.backward();

        let grad_x = x.grad();
        let grad_y = y.grad();
        let expected_grad_x = y.sin().value();
        let expected_grad_y = (x.value() + y.value()) * y.value().cos() + y.value().sin();

        assert!((grad_x - expected_grad_x).abs() < 1e-6);
        assert!((grad_y - expected_grad_y).abs() < 1e-6);
    }

    #[test]
    fn test_combined_operations2() {
        // Test case from `Modern Computational Finance: AAD and Parallel Simulations` by Antoine Savine
        const MULTIPLIER: f64 = 5.0;
        const TOLERANCE: f64 = 1e-6;

        let tape = Tape::default();

        let x0 = tape.var(1.0);
        let x1 = tape.var(2.0);
        let x2 = tape.var(3.0);
        let x3 = tape.var(4.0);
        let x4 = tape.var(5.0);

        let y1 = x2 * ((MULTIPLIER * x0) + x1);
        let y2 = y1.ln();
        let y = (y1 + (x3 * y2)) * (y1 + y2);

        y.backward();

        let expected_gradients = [
            950.7364539019619,  // grad_x0
            190.14729078039238, // grad_x1
            443.6770118209156,  // grad_x2
            73.20408806599326,  // grad_x3
            0.0,                // grad_x4
        ];

        let gradients = [x0.grad(), x1.grad(), x2.grad(), x3.grad(), x4.grad()];

        for (i, (&actual, &expected)) in gradients.iter().zip(&expected_gradients).enumerate() {
            assert!(
                (actual - expected).abs() < TOLERANCE,
                "Gradient mismatch for x{}: expected {}, got {}",
                i,
                expected,
                actual
            );
        }
    }
}
