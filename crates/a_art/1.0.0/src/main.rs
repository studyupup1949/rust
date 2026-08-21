
use a_art::PrimaryColor;
use a_art::mix;

/// version 1.0.0
fn main() {
    let red = PrimaryColor::Red;
    let yellow = PrimaryColor::Yellow;
    let result = mix(red, yellow);

    println!("{:?}", result);
}
