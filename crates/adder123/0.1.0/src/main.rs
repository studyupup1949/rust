use add_one123;

fn main() {
    let num = 10;

    println!("Hello, world! {} plus one is {}!",
        num,
        add_one123::add_one(num)
    );
}
