use access_path::KeyPath;

#[derive(KeyPath)]
enum Wrong {
    X(i32),
}

fn main() {}
