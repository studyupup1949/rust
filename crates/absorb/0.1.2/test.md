# The Rust Programming Language: A Practical Guide

Rust is a systems programming language focused on safety, speed, and concurrency. It
accomplishes these goals without requiring a garbage collector, making it a useful language
for a number of use cases other languages are not well suited for: embedding in other
languages, programs with specific space and time requirements, and writing low-level code
such as device drivers and operating systems.

## Why Rust?

The Rust programming language was originally created by Graydon Hoare at Mozilla Research
in 2010. It has since grown into one of the most beloved programming languages in the world,
consistently topping the Stack Overflow Developer Survey as the most admired language for
multiple years running. But what makes Rust so special?

At its core, Rust provides a unique combination of performance and safety. Traditional
systems programming languages like C and C++ give you fine-grained control over memory and
hardware, but they leave you responsible for avoiding undefined behavior, data races, and
memory leaks. Higher-level languages like Python and JavaScript manage memory for you, but
at the cost of runtime performance and control. Rust bridges this gap by enforcing memory
safety at compile time through its ownership system.

## Getting Started

To install Rust, use `rustup`, the official Rust toolchain installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once installed, verify your installation:

```bash
rustc --version
cargo --version
```

Create a new project:

```bash
cargo new my_project
cd my_project
cargo run
```

## Ownership and Borrowing

Ownership is Rust's most unique feature and has deep implications for the rest of the
language. It enables Rust to make memory safety guarantees without needing a garbage
collector. Understanding ownership is essential to writing idiomatic Rust.

### The Rules of Ownership

There are three fundamental rules of ownership in Rust:

1. Each value in Rust has a single owner.
2. There can only be one owner at a time.
3. When the owner goes out of scope, the value will be dropped.

Here is an example that demonstrates ownership transfer, also known as a move:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // s1 is moved to s2

    // println!("{}", s1); // This would cause a compile error!
    println!("{}", s2); // This works fine
}
```

### Borrowing

Instead of transferring ownership, you can borrow a reference to a value. References allow
you to refer to a value without taking ownership of it. There are two kinds of references:
immutable references (`&T`) and mutable references (`&mut T`).

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}

fn main() {
    let s = String::from("hello");
    let len = calculate_length(&s);
    println!("The length of '{}' is {}.", s, len);
}
```

The borrowing rules are as follows:

- At any given time, you can have either one mutable reference or any number of immutable
  references.
- References must always be valid (no dangling references).

These rules are enforced at compile time by the borrow checker, which is one of the most
powerful features of the Rust compiler.

## Data Types

Rust has a rich type system. Here is a comparison of the most commonly used primitive types:

| Type   | Size (bytes) | Range / Description             | Default |
| ------ | -----------: | ------------------------------- | ------- |
| `i8`   |            1 | -128 to 127                     | 0       |
| `i16`  |            2 | -32,768 to 32,767               | 0       |
| `i32`  |            4 | -2,147,483,648 to 2,147,483,647 | 0       |
| `i64`  |            8 | -9.2e18 to 9.2e18               | 0       |
| `i128` |           16 | -1.7e38 to 1.7e38               | 0       |
| `u8`   |            1 | 0 to 255                        | 0       |
| `u16`  |            2 | 0 to 65,535                     | 0       |
| `u32`  |            4 | 0 to 4,294,967,295              | 0       |
| `u64`  |            8 | 0 to 1.8e19                     | 0       |
| `u128` |           16 | 0 to 3.4e38                     | 0       |
| `f32`  |            4 | IEEE 754 single precision       | 0.0     |
| `f64`  |            8 | IEEE 754 double precision       | 0.0     |
| `bool` |            1 | true or false                   | false   |
| `char` |            4 | Unicode scalar value            | —       |
| `()`   |            0 | Unit type (empty tuple)         | ()      |

## Structs and Enums

Structs let you create custom data types that group related values together:

```rust
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

fn main() {
    let origin = Point::new(0.0, 0.0);
    let p = Point::new(3.0, 4.0);
    println!("Distance: {}", origin.distance_to(&p)); // 5.0
}
```

Enums are even more powerful in Rust than in most languages. Each variant can carry
different types and amounts of data:

```rust
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle { base: f64, height: f64 },
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }
}

fn main() {
    let shapes: Vec<Shape> = vec![
        Shape::Circle(5.0),
        Shape::Rectangle(4.0, 6.0),
        Shape::Triangle { base: 3.0, height: 8.0 },
    ];

    for shape in &shapes {
        println!("Area: {:.2}", shape.area());
    }
}
```

## Error Handling

Rust does not have exceptions. Instead, it uses the `Result<T, E>` type for recoverable
errors and the `panic!` macro for unrecoverable errors. This forces you to handle errors
explicitly at every call site.

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_file_contents(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    match read_file_contents("hello.txt") {
        Ok(contents) => println!("{}", contents),
        Err(e) => eprintln!("Failed to read file: {}", e),
    }
}
```

The `?` operator is syntactic sugar that propagates errors up the call stack. It is one of
the most frequently used features in Rust codebases.

### Common Error Handling Patterns

| Pattern                | Use Case                          | Example                          |
| ---------------------- | --------------------------------- | -------------------------------- | --- | ------- |
| `match`                | Handle each variant explicitly    | `match result { Ok(v) => ... }`  |
| `?` operator           | Propagate errors to the caller    | `let val = operation()?;`        |
| `unwrap()`             | Crash if error (prototyping only) | `result.unwrap()`                |
| `expect("msg")`        | Crash with message if error       | `result.expect("failed")`        |
| `unwrap_or(default)`   | Use a default value on error      | `result.unwrap_or(0)`            |
| `unwrap_or_else(f)`    | Compute fallback lazily           | `result.unwrap_or_else(          | \_  | 0)`     |
| `map()` / `and_then()` | Transform contained values        | `result.map(                     | v   | v + 1)` |
| `if let Ok(v) = expr`  | Handle only the success case      | `if let Ok(v) = parse() { ... }` |

## Traits

Traits define shared behavior. They are similar to interfaces in other languages but more
powerful because they support default implementations and can be implemented for any type
including foreign types (subject to the orphan rule).

```rust
trait Summary {
    fn summarize_author(&self) -> String;

    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

struct Article {
    title: String,
    author: String,
    content: String,
}

impl Summary for Article {
    fn summarize_author(&self) -> String {
        self.author.clone()
    }

    fn summarize(&self) -> String {
        format!("{}, by {} — {}", self.title, self.author, &self.content[..50])
    }
}

struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
}
```

### Common Standard Library Traits

| Trait           | Purpose                                 | Auto-derive? | Example Method         |
| --------------- | --------------------------------------- | :----------: | ---------------------- |
| `Debug`         | Debug formatting                        |     Yes      | `fmt(&self, f)`        |
| `Clone`         | Explicit deep copy                      |     Yes      | `clone(&self) -> Self` |
| `Copy`          | Implicit bitwise copy                   |     Yes      | (marker trait)         |
| `PartialEq`     | Equality comparison                     |     Yes      | `eq(&self, other)`     |
| `Eq`            | Total equality (reflexive)              |     Yes      | (marker trait)         |
| `PartialOrd`    | Partial ordering                        |     Yes      | `partial_cmp()`        |
| `Ord`           | Total ordering                          |     Yes      | `cmp()`                |
| `Hash`          | Hashing for hash maps and sets          |     Yes      | `hash()`               |
| `Default`       | Default value                           |     Yes      | `default() -> Self`    |
| `Display`       | User-facing formatting                  |      No      | `fmt(&self, f)`        |
| `Iterator`      | Iteration protocol                      |      No      | `next() -> Option<T>`  |
| `From` / `Into` | Type conversion                         |      No      | `from(val) -> Self`    |
| `Drop`          | Destructor logic                        |      No      | `drop(&mut self)`      |
| `Deref`         | Smart pointer dereferencing             |      No      | `deref(&self) -> &T`   |
| `Send`          | Safe to transfer across threads         |     Auto     | (marker trait)         |
| `Sync`          | Safe to reference from multiple threads |     Auto     | (marker trait)         |

## Iterators and Closures

Rust iterators are lazy and composable. They form the backbone of idiomatic Rust data
processing. Combined with closures, they allow you to express complex transformations in a
concise and efficient way.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Sum of squares of even numbers
    let sum: i32 = numbers
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .sum();

    println!("Sum of squares of evens: {}", sum); // 220

    // Collect into a new collection
    let doubled: Vec<i32> = numbers.iter().map(|&x| x * 2).collect();
    println!("Doubled: {:?}", doubled);

    // Find the first number greater than 7
    let found = numbers.iter().find(|&&x| x > 7);
    println!("First > 7: {:?}", found); // Some(8)

    // Chain multiple iterators
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let chained: Vec<&i32> = a.iter().chain(b.iter()).collect();
    println!("Chained: {:?}", chained);
}
```

### Iterator Adapter Reference

| Adapter         | Description                       | Lazy? | Returns     |
| --------------- | --------------------------------- | :---: | ----------- |
| `map(f)`        | Transform each element            |  Yes  | Iterator    |
| `filter(p)`     | Keep elements matching predicate  |  Yes  | Iterator    |
| `enumerate()`   | Attach index to each element      |  Yes  | Iterator    |
| `zip(other)`    | Pair elements from two iterators  |  Yes  | Iterator    |
| `chain(other)`  | Concatenate two iterators         |  Yes  | Iterator    |
| `take(n)`       | Take first n elements             |  Yes  | Iterator    |
| `skip(n)`       | Skip first n elements             |  Yes  | Iterator    |
| `flatten()`     | Flatten nested iterators          |  Yes  | Iterator    |
| `peekable()`    | Allow peeking at the next element |  Yes  | Peekable    |
| `rev()`         | Reverse a double-ended iterator   |  Yes  | Iterator    |
| `collect()`     | Consume into a collection         |  No   | Collection  |
| `sum()`         | Sum all elements                  |  No   | Scalar      |
| `count()`       | Count elements                    |  No   | usize       |
| `any(p)`        | True if any element matches       |  No   | bool        |
| `all(p)`        | True if all elements match        |  No   | bool        |
| `find(p)`       | First element matching predicate  |  No   | Option      |
| `fold(init, f)` | Reduce to a single value          |  No   | Accumulator |

## Concurrency

Rust's ownership model naturally extends to concurrent programming. The compiler prevents
data races at compile time, making concurrent Rust programs both fast and safe.

### Threads

```rust
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap()); // 10
}
```

### Channels

Channels provide a way to send messages between threads. Rust's standard library provides
a multi-producer, single-consumer channel:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    let tx1 = tx.clone();
    thread::spawn(move || {
        let messages = vec!["hello", "from", "thread", "one"];
        for msg in messages {
            tx1.send(msg).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
    });

    thread::spawn(move || {
        let messages = vec!["more", "messages", "from", "thread", "two"];
        for msg in messages {
            tx.send(msg).unwrap();
            thread::sleep(Duration::from_millis(150));
        }
    });

    for received in rx {
        println!("Got: {}", received);
    }
}
```

### Concurrency Primitives Comparison

| Primitive            | Use Case                         | Thread-safe? | Blocking? |
| -------------------- | -------------------------------- | :----------: | :-------: |
| `Mutex<T>`           | Shared mutable state             |     Yes      |    Yes    |
| `RwLock<T>`          | Many readers, few writers        |     Yes      |    Yes    |
| `Arc<T>`             | Shared ownership across threads  |     Yes      |    No     |
| `mpsc::channel`      | Message passing (multi-producer) |     Yes      |    Yes    |
| `mpsc::sync_channel` | Bounded message passing          |     Yes      |    Yes    |
| `AtomicBool`         | Lock-free boolean flag           |     Yes      |    No     |
| `AtomicUsize`        | Lock-free counter                |     Yes      |    No     |
| `Barrier`            | Synchronize multiple threads     |     Yes      |    Yes    |
| `Condvar`            | Wait for a condition             |     Yes      |    Yes    |
| `Once`               | One-time initialization          |     Yes      |    No     |

## Lifetimes

Lifetimes are Rust's way of ensuring that references are always valid. Most of the time,
lifetimes are inferred by the compiler. But sometimes you need to annotate them explicitly
to help the compiler understand the relationship between references.

```rust
// This function returns a reference to the longer of two string slices.
// The lifetime annotation 'a tells the compiler that the returned reference
// will be valid for at least as long as both input references.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("long string is long");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(string1.as_str(), string2.as_str());
        println!("The longest string is '{}'", result);
    }
    // Note: using `result` here would be a compile error because
    // string2 has been dropped and result might reference it.
}
```

### Lifetime Elision Rules

The compiler applies three rules to figure out lifetimes when they are not explicitly
annotated:

1. Each parameter that is a reference gets its own lifetime parameter.
2. If there is exactly one input lifetime parameter, that lifetime is assigned to all
   output lifetime parameters.
3. If there is a `&self` or `&mut self` parameter, the lifetime of self is assigned to
   all output lifetime parameters.

## Pattern Matching

Pattern matching with `match` is one of Rust's most expressive features. It is exhaustive,
meaning the compiler ensures you handle every possible case.

```rust
enum Command {
    Quit,
    Echo(String),
    Move { x: i32, y: i32 },
    ChangeColor(u8, u8, u8),
}

fn process_command(cmd: Command) {
    match cmd {
        Command::Quit => {
            println!("Quitting");
        }
        Command::Echo(msg) => {
            println!("Echo: {}", msg);
        }
        Command::Move { x, y } => {
            println!("Moving to ({}, {})", x, y);
        }
        Command::ChangeColor(r, g, b) => {
            println!("Changing color to ({}, {}, {})", r, g, b);
        }
    }
}

fn describe_number(n: i32) -> &'static str {
    match n {
        0 => "zero",
        1..=9 => "single digit",
        10..=99 => "double digits",
        100..=999 => "triple digits",
        _ if n < 0 => "negative",
        _ => "very large",
    }
}

fn main() {
    process_command(Command::Echo("hello world".to_string()));
    process_command(Command::Move { x: 10, y: 20 });
    process_command(Command::ChangeColor(255, 128, 0));
    process_command(Command::Quit);

    for n in [-5, 0, 7, 42, 100, 10000] {
        println!("{}: {}", n, describe_number(n));
    }
}
```

## Smart Pointers

Rust provides several smart pointer types in the standard library that go beyond ordinary
references by providing additional functionality.

| Smart Pointer | Heap Alloc? | Shared? | Mutable? | Thread-safe? | Use Case                         |
| ------------- | :---------: | :-----: | :------: | :----------: | -------------------------------- |
| `Box<T>`      |     Yes     |   No    |   Yes    |  If T: Send  | Heap allocation, recursive types |
| `Rc<T>`       |     Yes     |   Yes   |    No    |      No      | Single-threaded shared ownership |
| `Arc<T>`      |     Yes     |   Yes   |    No    |     Yes      | Multi-threaded shared ownership  |
| `Cell<T>`     |     No      |   No    |   Yes    |      No      | Interior mutability (Copy types) |
| `RefCell<T>`  |     No      |   No    |   Yes    |      No      | Interior mutability (runtime)    |
| `Mutex<T>`    |     No      |   No    |   Yes    |     Yes      | Thread-safe interior mutability  |
| `Cow<'a, B>`  |    Maybe    |   No    |    No    |  If B: Send  | Clone-on-write optimization      |

```rust
use std::rc::Rc;
use std::cell::RefCell;

// A simple tree structure using Rc and RefCell
#[derive(Debug)]
struct Node {
    value: i32,
    children: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(value: i32) -> Rc<RefCell<Node>> {
        Rc::new(RefCell::new(Node {
            value,
            children: vec![],
        }))
    }

    fn add_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
        parent.borrow_mut().children.push(child);
    }
}

fn main() {
    let root = Node::new(1);
    let child_a = Node::new(2);
    let child_b = Node::new(3);
    let grandchild = Node::new(4);

    Node::add_child(&root, Rc::clone(&child_a));
    Node::add_child(&root, Rc::clone(&child_b));
    Node::add_child(&child_a, Rc::clone(&grandchild));

    println!("Tree: {:#?}", root.borrow());
}
```

## Collections

Rust's standard library provides several useful collection types. Here is a summary:

| Collection       | Ordered? | Unique Keys? |  Lookup  |  Insert   | Use Case                 |
| ---------------- | :------: | :----------: | :------: | :-------: | ------------------------ |
| `Vec<T>`         |   Yes    |      No      | O(1) idx | O(1) end  | General-purpose sequence |
| `VecDeque<T>`    |   Yes    |      No      | O(1) idx | O(1) ends | Double-ended queue       |
| `LinkedList<T>`  |   Yes    |      No      |   O(n)   | O(1) ends | Rarely used in practice  |
| `HashMap<K, V>`  |    No    |     Yes      | O(1) avg | O(1) avg  | Key-value mapping        |
| `BTreeMap<K, V>` |   Yes    |     Yes      | O(log n) | O(log n)  | Sorted key-value mapping |
| `HashSet<T>`     |    No    |     Yes      | O(1) avg | O(1) avg  | Unique values            |
| `BTreeSet<T>`    |   Yes    |     Yes      | O(log n) | O(log n)  | Sorted unique values     |
| `BinaryHeap<T>`  | Partial  |      No      | O(1) max | O(log n)  | Priority queue           |

```rust
use std::collections::HashMap;

fn word_count(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        let count = counts.entry(word).or_insert(0);
        *count += 1;
    }
    counts
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog the fox";
    let counts = word_count(text);

    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    for (word, count) in sorted {
        println!("{:>8}: {}", word, count);
    }
}
```

## Async Rust

Asynchronous programming in Rust is built around the `Future` trait and the `async/await`
syntax. The standard library provides the core primitives but you typically need an async
runtime like `tokio` or `async-std` to actually execute futures.

```rust
use tokio::time::{sleep, Duration};
use tokio::task;

async fn fetch_data(id: u32) -> String {
    // Simulate an async operation
    sleep(Duration::from_millis(100 * id as u64)).await;
    format!("Data from source {}", id)
}

#[tokio::main]
async fn main() {
    // Run multiple futures concurrently
    let (r1, r2, r3) = tokio::join!(
        fetch_data(1),
        fetch_data(2),
        fetch_data(3),
    );

    println!("{}", r1);
    println!("{}", r2);
    println!("{}", r3);

    // Spawn tasks for true parallelism
    let mut handles = vec![];
    for i in 0..5 {
        handles.push(task::spawn(async move {
            fetch_data(i).await
        }));
    }

    for handle in handles {
        let result = handle.await.unwrap();
        println!("{}", result);
    }
}
```

## Cargo and the Ecosystem

Cargo is Rust's build system and package manager. It handles downloading dependencies,
compiling your code, running tests, generating documentation, and much more. Here are some
of the most useful Cargo commands:

| Command                 | Description                           |
| ----------------------- | ------------------------------------- |
| `cargo new <name>`      | Create a new project                  |
| `cargo build`           | Compile the project                   |
| `cargo build --release` | Compile with optimizations            |
| `cargo run`             | Build and run the project             |
| `cargo test`            | Run all tests                         |
| `cargo test <name>`     | Run tests matching a name             |
| `cargo bench`           | Run benchmarks                        |
| `cargo doc --open`      | Generate and open documentation       |
| `cargo clippy`          | Run the Rust linter                   |
| `cargo fmt`             | Format code with rustfmt              |
| `cargo check`           | Type-check without producing a binary |
| `cargo update`          | Update dependencies                   |
| `cargo publish`         | Publish a crate to crates.io          |
| `cargo install <crate>` | Install a binary crate                |
| `cargo add <crate>`     | Add a dependency to Cargo.toml        |

A typical `Cargo.toml` looks like this:

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A brief description of the project"
license = "MIT"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
anyhow = "1.0"

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"

[[bench]]
name = "my_benchmark"
harness = false
```

## Testing

Rust has first-class support for testing built right into the language and tooling. Unit
tests live alongside the code they test, while integration tests go in a separate `tests`
directory.

```rust
/// Adds two numbers together.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Divides two numbers, returning None if the divisor is zero.
pub fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, -1), -2);
    }

    #[test]
    fn test_add_zero() {
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_divide_normal() {
        assert_eq!(divide(10.0, 2.0), Some(5.0));
    }

    #[test]
    fn test_divide_by_zero() {
        assert_eq!(divide(10.0, 0.0), None);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_overflow() {
        let _ = (i32::MAX).checked_add(1).expect("overflow");
    }
}
```

## Macros

Macros are a powerful metaprogramming feature in Rust. Declarative macros (`macro_rules!`)
allow you to write code that writes code, reducing boilerplate and enabling domain-specific
patterns.

```rust
macro_rules! vec_of_strings {
    ($($x:expr),* $(,)?) => {
        vec![$($x.to_string()),*]
    };
}

macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert($key, $value);)*
        map
    }};
}

fn main() {
    let names = vec_of_strings!["Alice", "Bob", "Charlie"];
    println!("{:?}", names);

    let scores = hashmap! {
        "Alice" => 95,
        "Bob" => 87,
        "Charlie" => 92,
    };
    println!("{:?}", scores);
}
```

## Unsafe Rust

Sometimes you need to step outside the safety guarantees of the Rust compiler. Unsafe Rust
gives you access to five additional capabilities:

1. Dereference raw pointers
2. Call unsafe functions or methods
3. Access or modify mutable static variables
4. Implement unsafe traits
5. Access fields of unions

```rust
fn main() {
    let mut value: i32 = 42;

    // Create raw pointers (this is safe)
    let r1 = &value as *const i32;
    let r2 = &mut value as *mut i32;

    // Dereferencing raw pointers requires unsafe
    unsafe {
        println!("r1 points to: {}", *r1);
        *r2 = 100;
        println!("r2 changed value to: {}", *r2);
    }

    // Calling an unsafe function
    unsafe {
        let layout = std::alloc::Layout::new::<[u8; 1024]>();
        let ptr = std::alloc::alloc(layout);
        if !ptr.is_null() {
            std::ptr::write_bytes(ptr, 0, 1024);
            std::alloc::dealloc(ptr, layout);
        }
    }
}
```

The key principle is to keep `unsafe` blocks as small as possible and to wrap them in safe
abstractions. The standard library itself is full of carefully audited unsafe code that
exposes safe interfaces.

## Conclusion

Rust represents a fundamental shift in how we think about systems programming. By moving
safety checks from runtime to compile time, it eliminates entire categories of bugs while
maintaining the performance characteristics that systems programmers need. The initial
learning curve is steeper than many other languages, but the payoff in reliability and
maintainability is substantial.

Whether you are building web servers, embedded systems, command-line tools, game engines,
or operating systems, Rust gives you the tools to write correct, efficient, and maintainable
code. The ecosystem continues to grow rapidly, and the community remains one of the most
welcoming in the programming world.
