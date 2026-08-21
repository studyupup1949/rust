# advent-of-rust

This crate contains the `advent_of_rust!` macro, a utility macro which makes it easy to run advent of code challenges with minimal boilerplate.

Each Advent of Code day is a module called `day_N` with two functions:

- `part_1`
- `part_2` (optional)

This minimal example will test day `4` which has a `part_2` and `part_1`.

Filename: `src/day_4.rs`.

```rust ignore
pub fn part_1(input: &str) -> u64 {
    99999
}

pub fn part_2(input: &str) -> &str {
    "Advent of Rust!"
}
```

Filename: `src/lib.rs`.

```rust ignore
advent_of_rust::advent_of_rust! {
  4 => 99999, "Advent of Rust!";
}
```

The above will expand to roughly this code:

```rust ignore
mod day_4;

#[cfg(test)]
mod tests {
    use super::*;

    const DAY_4_INPUT: &str = include_str!("day_4.txt");

    #[test]
    fn day_4_part_1() {
        assert_eq!(day_4::part_1(DAY_4_INPUT), 99999);
    }

    #[test]
    fn day_4_part_2() {
        assert_eq!(day_4::part_2(DAY_4_INPUT), "Advent of Rust!");
    }
}
```

Then you can test your advent of code:

You put your puzzle input into a file called `day_N.txt` which lives in the same directory where you invoked the `advent_of_rust!` macro.

```text
$ cargo test

running 2 tests
test tests::day_4_part_1 ... ok
test tests::day_4_part_2 ... ok
```

Each module _must_ have a `part_1` function and may also have a `part_2` as well.

An example `src/lib.rs` with multiple days:

```rust ignore
advent_of_rust::advent_of_rust! {
  1 => 1234, "Advent of Rust!";
  22 => "hello";
  3 => 1234;
  4 => Err(4), false;
  8 => 1234, None;
}
```

For the above to compile you will need the following in `src/` dir:

- `day_1.rs` with `fn part_1(&str) -> u16` and `fn part_2(&str) -> &str` and `day_1.txt` .
- `day_22.rs` with `fn part_1(&str) -> u16` and `day_22.txt`.
- `day_3.rs` with `fn part_1(&str) -> u16` and `day_3.txt` .
- `day_4.rs` with `fn part_1(&str) -> u16` and `fn part_2(&str) -> &str` and `day_4.txt` .
- `day_8.rs` with `fn part_1(&str) -> u16` and `fn part_2(&str) -> &str` and `day_8.txt` .
