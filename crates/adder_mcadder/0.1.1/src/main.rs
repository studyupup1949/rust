/// Adds one to the number given.
///
/// # Examples
///
/// ```
/// let arg = 5;
/// let answer = my_crate::add_one(arg);
///
/// assert_eq!(7, answer);
/// ```

pub fn add_one(x: i32) -> i32 {
    x + 1
}
fn main() {
    let x = 3;
    let y = add_one(x);
    println!("The input is {}, the output is {}", x, y);
}
