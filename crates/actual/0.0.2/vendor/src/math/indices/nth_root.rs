// use num_traits::Float;
// /// Computes number^(1/root_power) via Newton's method.
// pub fn nth_root<T: Float>(number: T, root_power: T) -> T {
//     let zero = T::zero();
//     let one = T::one();

//     if number < zero || root_power <= zero {
//         return T::nan();
//     }
//     if number == zero {
//         return zero;
//     }

//     let mut guess = number.max(one); // safe starting point for any positive number
//     let epsilon = T::epsilon() * T::from(100).unwrap();

//     loop {
//         let delta = (number / guess.powf(root_power - one) - guess) / root_power;
//         guess = guess + delta;
//         if delta.abs() < epsilon {
//             break;
//         }
//     }

//     guess
// }

// Omg this sentire func is soo stupid.
// WE ALWAYS GONNA USE A F64. Fuh u MEAN REUTNR u32 IF ENTERD u32?????

// :wilt

// let me rewrie
use num_traits::Float;

/// Computes number^(1/root_power) via Newton's method.
pub fn nth_root<T: Float>(number: T, root_power: T) -> T {
    let zero = T::zero();
    let one = T::one();

    if number < zero || root_power <= zero {
        return T::nan();
    }
    if number == zero {
        return zero;
    }

    let mut guess = number.max(one); // safe starting point for any positive number
    let epsilon = T::epsilon() * T::from(100).unwrap();

    loop {
        let delta = (number / guess.powf(root_power - one) - guess) / root_power;
        guess = guess + delta;
        if delta.abs() < epsilon {
            break;
        }
    }

    guess
}

// fn main() {
//     println!("{}", nth_root(8.0_f64, 3.0)); // ~2.0
//     println!("{}", nth_root(2.0_f64, 2.0)); // ~1.4142...
//     println!("{}", nth_root(0.5_f64, 2.0)); // ~0.7071...
// }

fn nth_roots(a: f64, _b: f64) -> f64 {
    let n = a; // 27.
    let mut guess_new = n / 2.0;
    let mut unknown = n / guess_new.powf(n - 1.0);
    let mut mean = (guess_new * (n - 1.0) + unknown) / n;
    let mut difference = (guess_new - mean).abs();
    let mut count = 0;
    loop {
        if difference <= 0.00000000001 && count >= 10 {
            break;
        }
        guess_new = mean;
        unknown = n / guess_new.powf(n - 1.0);
        mean = (guess_new * (n - 1.0) + unknown) / n;
        difference = (guess_new - mean).abs();
        guess_new = mean;
        count += 1;
        println!(
            "count = {count} \n, difference = {difference}, \n unknown = {unknown}\n, mean = {mean}, guess_new = {guess_new}"
        );
        guess_new = mean;
    }
    guess_new
}
