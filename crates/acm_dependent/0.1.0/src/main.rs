extern crate arithmetic_congruence_monoid;

use arithmetic_congruence_monoid::acm::calculate_acm;
use arithmetic_congruence_monoid::acm2::calculate_density;

fn main() {
    let a = 1;
    let b = 4;
    let acm_elements = calculate_acm(a, b);
    println!("Elements of ACM: {:?}", acm_elements);

    let density = calculate_density();
    println!("Atomic density: {}", density);
}
