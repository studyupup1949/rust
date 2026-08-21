
use a_art::PrimaryColor;
use a_art::mix;


fn main() {
    let red = PrimaryColor::Red;
    let yellow = PrimaryColor::Yellow;
    let result = mix(red, yellow);

    println!("{:?}", result);
}
