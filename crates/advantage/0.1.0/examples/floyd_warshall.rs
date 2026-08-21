extern crate advantage as adv;

use adv::prelude::*;

adv_struct! {
    /// State of a floyd warshall execution
    #[derive(Debug, Clone)]
    struct FloydWarshallState {
        pub n: usize,
        pub k: usize,
        pub dmat: Vec<Scalar>,
    }
}

adv_fn! {
    /// Initialize a floyd warshall matrix
    fn floyd_warshall_init(n: usize, inf: f64) -> FloydWarshallState<Scalar> {
        let mut state = FloydWarshallState {
            n,
            k: 0,
            dmat: vec![inf.into(); n*n],
        };
        for i in 0..n {
            state.dmat[i*n + i] = 0.0.into();
        }
        state
    }
}

adv_fn! {
    /// Declare an edge in the floyd warshall matrix
    fn floyd_warshall_set_edge(state: &mut FloydWarshallState<Scalar>, u: usize, v: usize, weight: Scalar) {
        let n = state.n;
        state.dmat[u*n + v] = weight;
        state.dmat[v*n + u] = weight;
    }
}

adv_fn! {
    /// Declare an edge in the floyd warshall matrix
    fn floyd_warshall_get_edge(state: &FloydWarshallState<Scalar>, u: usize, v: usize) -> Scalar {
        let n = state.n;
        state.dmat[u*n + v]
    }
}

adv_fn! {
    /// Single step of the floyd warshall algorithm
    fn floyd_warshall_step(state: FloydWarshallState<Scalar>) -> FloydWarshallState<Scalar> {
        let mut state = state;
        let n = state.n;
        let k = state.k;
        for i in 0..n {
            for j in 0..n {
                state.dmat[i*n + j] = state.dmat[i*n + j].min(state.dmat[i*n + k] + state.dmat[k*n + j]);
            }
        }
        state.k += 1;
        state
    }
}

fn main() {
    let n = 23;
    let inf = 10e3;

    // We have a big intersection of 5 different streets looking like this:
    //
    //      0     1
    //      |     |
    // 2 ---3--4--5---6
    //      |     |
    //      7     8
    //      |     |
    // 9 ---10-11-12--13
    //      |     |
    //      14    15
    //      |     |
    // 16---17-18-19--20
    //      |     |
    //      21    22
    //
    // Each intersection has a traffic light which allows entering the intersection only vertically
    // or horizontally at any given time.
    adv_struct! {
        struct TrafficLightParams {
            pub top_bottom_quota: Scalar,
            pub left_right_quota: Scalar,
        }
    }

    // Of course quotas add up to 1.0 but for numerical stability reasons they may not be exactly
    // 0.0 or 1.0 alone. We can represent this using only 3 parameters since the 4th follows from
    // the other 3.
    adv_fn! {
        fn new_traffic_light(top_bottom_quota: Scalar) -> TrafficLightParams<Scalar> {
            let top_bottom_quota = top_bottom_quota.max(0.01.into()).min(0.99.into());
            let left_right_quota = 1.0 - top_bottom_quota;
            TrafficLightParams {
                top_bottom_quota,
                left_right_quota,
            }
        }
    }

    let mut ctx = adv::AContext::new();
    {
        // We have 6 traffic lights
        let tl3 = new_traffic_light(ctx.new_indep(0.0));
        let tl5 = new_traffic_light(ctx.new_indep(0.0));
        let tl10 = new_traffic_light(ctx.new_indep(0.0));
        let tl12 = new_traffic_light(ctx.new_indep(0.0));
        let tl17 = new_traffic_light(ctx.new_indep(0.0));
        let tl19 = new_traffic_light(ctx.new_indep(0.0));

        // We model the graph using the variables from the traffic lights
        let mut graph = floyd_warshall_init(n, inf);
        floyd_warshall_set_edge(&mut graph, 0, 3, 1.0 + 1.0 / tl3.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 1, 5, 1.0 + 1.0 / tl5.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 2, 3, 1.0 + 1.0 / tl3.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 3, 4, 1.0 + 1.0 / tl3.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 4, 5, 1.0 + 1.0 / tl5.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 5, 6, 1.0 + 1.0 / tl5.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 3, 7, 1.0 + 1.0 / tl3.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 5, 8, 1.0 + 1.0 / tl3.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 7, 10, 1.0 + 1.0 / tl10.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 8, 12, 1.0 + 1.0 / tl12.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 9, 10, 1.0 + 1.0 / tl10.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 10, 11, 1.0 + 1.0 / tl10.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 11, 12, 1.0 + 1.0 / tl12.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 12, 13, 1.0 + 1.0 / tl12.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 10, 14, 1.0 + 1.0 / tl10.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 12, 15, 1.0 + 1.0 / tl10.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 14, 17, 1.0 + 1.0 / tl17.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 15, 19, 1.0 + 1.0 / tl19.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 16, 17, 1.0 + 1.0 / tl17.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 17, 18, 1.0 + 1.0 / tl17.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 18, 19, 1.0 + 1.0 / tl19.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 19, 20, 1.0 + 1.0 / tl19.left_right_quota);
        floyd_warshall_set_edge(&mut graph, 17, 21, 1.0 + 1.0 / tl17.top_bottom_quota);
        floyd_warshall_set_edge(&mut graph, 19, 22, 1.0 + 1.0 / tl17.top_bottom_quota);
        ctx.set_dep_slice(&graph.dmat[..]);
    }
    let tape_init = ctx.tape();

    fn tape_step(n: usize, k: usize) -> impl adv::Tape + Clone {
        let mut ctx = adv::AContext::new();
        let input = FloydWarshallState {
            n,
            k,
            dmat: ctx.new_indep_vec(n * n, 0.0),
        };
        let output = floyd_warshall_step(input);
        ctx.set_dep_slice(&output.dmat);
        ctx.tape()
    }

    adv_fn! {
        /// We want the sum of routes between exits to be minimal
        fn badness(graph: &FloydWarshallState<Scalar>) -> Scalar {
            let exits = vec![0, 1, 2, 6, 9, 13, 16, 20, 21, 22];
            let mut result = Scalar::zero();
            for u in exits.iter().cloned() {
                for v in exits.iter().cloned() {
                    let edge = floyd_warshall_get_edge(graph, u, v);
                    result += edge*edge;
                }
            }
            result
        }
    }

    let mut ctx = adv::AContext::new();
    {
        let input = FloydWarshallState {
            n,
            k: 0,
            dmat: ctx.new_indep_vec(n * n, 0.0),
        };
        let error = badness(&input);
        ctx.set_dep(&error);
    }
    let tape_badness = ctx.tape();

    let zero_order = |params| {
        let mut tape_init = tape_init.clone();
        tape_init.zero_order(&params);
        let mut graph = tape_init.y();
        for k in 0..n {
            let mut tape_step = tape_step(n, k);
            tape_step.zero_order(&graph);
            graph = tape_step.y();
        }
        let mut tape_badness = tape_badness.clone();
        tape_badness.zero_order(&graph);
        tape_badness.y()[0]
    };

    let calculate_gradient = |params: &adv::DVector<f64>| {
        let forward = |(i, x)| {
            let mut tape: Box<dyn adv::Tape> = if i == 0 {
                Box::new(tape_init.clone())
            } else if i > 0 && i < (n + 1) {
                Box::new(tape_step(n, i - 1))
            } else {
                Box::new(tape_badness.clone())
            };
            tape.zero_order(&x);
            let y = tape.y();
            (i + 1, y)
        };
        let reverse = |(i, x), gen_jac_next| {
            let mut tape: Box<dyn adv::Tape> = if i == 0 {
                Box::new(tape_init.clone())
            } else if i > 0 && i < (n + 1) {
                Box::new(tape_step(n, i - 1))
            } else {
                Box::new(tape_badness.clone())
            };
            tape.zero_order(&x);
            adv::drivers::generalized_jacobian_tape(
                tape,
                &adv::DVector::from_element(x.nrows(), 0.0),
                &[0],
                Some(gen_jac_next),
            )
        };
        let identity = |(_, _)| adv::drivers::GeneralizedJacobian {
            homogenous: adv::DMatrix::identity(1, 1),
            inhomogenous: adv::DVector::zeros(1),
            multiplicity: 0,
        };
        adv::drivers::reverse_sequence((0, params.clone()), n + 2, n, forward, reverse, identity)
            .homogenous
            .transpose()
    };

    // We can optimize the quotas by calculating a gradient
    let mut params = adv::DVector::from_element(6, 0.5);
    println!("E = {}", zero_order(params.clone()).sqrt());
    let epsilon = 1e-10;
    let mut g = calculate_gradient(&params);
    println!("{}", g);
    let mut step = 0.00001;

    while g.norm() > epsilon {
        // Calculate new set of parameters and new gradient
        let params_new = &params - step * &g;
        println!("E = {}", zero_order(params.clone()).sqrt());
        let g_new = calculate_gradient(&params_new);
        println!("{}", g_new);

        // Calculate next step size
        let params_diff = &params_new - &params;
        let g_diff = &g_new - &g;
        step = ((&params_diff.transpose() * &g_diff).abs() / g_diff.norm_squared())[0];

        // Swap params and gradient
        params = params_new;
        g = g_new;
    }

    println!("quota = {}", params);
}
