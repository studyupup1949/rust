extern crate advantage as adv;

use adv::adv_fn;
use adv::prelude::*;

adv_fn! {
    /// Sigmoid function
    fn sigmoid(x: Scalar) -> Scalar {
        let x_exp = x.exp();
        x_exp / (x_exp + 1.0)
    }
}

adv_fn! {
    /// A simple multilayer perceptron with 2 input neuron, 2 hidden neurons and 1 output neuron
    fn multilayer_perceptron(
        inputs: &[f64],
        weights1: &[Scalar],
        biases1: &[Scalar],
        weights2: &[Scalar],
        biases2: &[Scalar],
    ) -> Scalar
    {
        assert_eq!(inputs.len(), 2);
        assert_eq!(weights1.len(), 4);
        assert_eq!(biases1.len(), 2);
        assert_eq!(weights2.len(), 2);
        assert_eq!(biases2.len(), 1);

        let hidden1 = sigmoid(inputs[0] * weights1[0] + inputs[1] * weights1[1] + biases1[0]);
        let hidden2 = sigmoid(inputs[0] * weights1[2] + inputs[1] * weights1[3] + biases1[1]);
        sigmoid(hidden1 * weights2[0] + hidden2 * weights2[1] + biases2[0])
    }
}

adv_fn! {
    /// Sum of square differences between the network and a xor-gate
    fn xor_error(weights1: &[Scalar], biases1: &[Scalar], weights2: &[Scalar], biases2: &[Scalar]) -> Scalar {
        assert_eq!(weights1.len(), 4);
        assert_eq!(biases1.len(), 2);
        assert_eq!(weights2.len(), 2);
        assert_eq!(biases2.len(), 1);

        let error1 = {
            let input = [0.0, 0.0];
            let expected = 0.0;
            let actual = multilayer_perceptron(&input, weights1, biases1, weights2, biases2);
            (actual - expected) * (actual - expected)
        };

        let error2 = {
            let input = [0.0, 1.0];
            let expected = 1.0;
            let actual = multilayer_perceptron(&input, weights1, biases1, weights2, biases2);
            (actual - expected) * (actual - expected)
        };

        let error3 = {
            let input = [1.0, 0.0];
            let expected = 1.0;
            let actual = multilayer_perceptron(&input, weights1, biases1, weights2, biases2);
            (actual - expected) * (actual - expected)
        };

        let error4 = {
            let input = [1.0, 1.0];
            let expected = 0.0;
            let actual = multilayer_perceptron(&input, weights1, biases1, weights2, biases2);
            (actual - expected) * (actual - expected)
        };

        error1 + error2 + error3 + error4
    }
}

fn gradient(tape: &mut dyn adv::Tape, params: &adv::DVector<f64>) -> adv::DMatrix<f64> {
    tape.zero_order(params);
    adv::drivers::jacobian_reverse(tape).transpose()
}

fn main() {
    // Tape the `xor_error` function
    let mut tape = {
        let mut ctx = adv::AContext::new();

        let mut parameters = Vec::with_capacity(9);
        for _ in 0..9 {
            parameters.push(ctx.new_indep(0.0));
        }
        let weights1 = &parameters[0..4];
        let biases1 = &parameters[4..6];
        let weights2 = &parameters[6..8];
        let biases2 = &parameters[8..];

        let error = xor_error(weights1, biases1, weights2, biases2);

        ctx.set_dep(&error);
        ctx.tape()
    };
    let n = tape.num_indeps();

    // Setup a initial values
    let mut params = adv::DVector::from_element(n, 0.0);

    // Input weigth initial values
    params[0] = -1.0;
    params[1] = -1.0;
    params[2] = 1.0;
    params[3] = 1.0;

    // Input bias initial values
    params[4] = 0.0;
    params[5] = 0.0;

    // Output weight initial values
    params[6] = 1.0;
    params[7] = 1.0;

    // Output bias initial value
    params[8] = 0.0;

    // Use a gradient descent on the parameters to minimize the error
    let epsilon = 1e-20;
    let mut g = gradient(&mut tape, &params);
    let mut step = 1.0;

    let mut iterations = 0;
    while g.norm() > epsilon {
        iterations += 1;

        // Calculate new set of parameters and new gradient
        let params_new = &params - step * &g;
        let g_new = gradient(&mut tape, &params_new);

        // Calculate next step size
        let params_diff = &params_new - &params;
        let g_diff = &g_new - &g;
        step = (&params_diff.transpose() * &g_diff)[0].abs() / &g_diff.norm_squared();

        // Swap params and gradient
        params = params_new;
        g = g_new;

        // Calculate error
        tape.zero_order(&params);
        let error = tape.y()[0];
        println!("E = {}", error);
    }
    println!("Finished in {} iterations", iterations);
    println!();

    let params = params.iter().cloned().collect::<Vec<f64>>();
    let weights1 = &params[0..4];
    let biases1 = &params[4..6];
    let weights2 = &params[6..8];
    let biases2 = &params[8..];

    // Print the parameters of the network
    println!("Input Weights  = {:?}", weights1);
    println!("Input Biases   = {:?}", biases1);
    println!("Output Weights = {:?}", weights2);
    println!("Output Biases  = {:?}", biases2);
    println!();

    // Print the 4 possible outputs of the trained network
    println!(
        "xor(0, 0) = {}",
        multilayer_perceptron(&[0.0, 0.0], weights1, biases1, weights2, biases2)
    );
    println!(
        "xor(0, 1) = {}",
        multilayer_perceptron(&[0.0, 1.0], weights1, biases1, weights2, biases2)
    );
    println!(
        "xor(1, 0) = {}",
        multilayer_perceptron(&[1.0, 0.0], weights1, biases1, weights2, biases2)
    );
    println!(
        "xor(1, 1) = {}",
        multilayer_perceptron(&[1.0, 1.0], weights1, biases1, weights2, biases2)
    );
}
