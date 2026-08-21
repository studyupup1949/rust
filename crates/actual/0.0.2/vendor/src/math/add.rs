#![allow(unused, non_camel_case_types)]

use std::ops::AddAssign;
pub fn add_two<T>(a: T, b: T) -> T
where
    T: std::ops::Add<Output = T>,
{
    a + b
}

// pub fn add_vectors<T>(a: Vec<T>, b: T) -> T
// where
//     T: std::ops,
// {
// }

// writeln!
pub fn sum_of_vector<T>(a: Vec<T>) -> T
where
    T: std::ops::Add<Output = T> + std::ops::AddAssign + Default,
{
    let mut count = T::default();
    for i in a {
        count += i;
    }
    count
}
pub struct Vector_Arithimatic<T> {
    pub vector: Vec<Vec<T>>,
}
pub fn add_vector<T>(a: Vec<Vec<T>>)
//-> Vec<T>
where
    T: AddAssign + std::ops::Add<Output = T> + num_traits::Num,
{
}

// pub fn add_vector<T>(a: Vec<Vec<T>>) -> Vec<T>
// where
//     T: std::ops::Add<

// pub fn add_vectors_index<
struct AddVector<T> {
    vector: Vec<Vec<T>>,
}
#[derive(Debug)]
enum Nested<T> {
    Value(T),
    List(Vec<Nested<T>>),
}

struct MyStruct<T> {
    data: Nested<T>,
}

fn main() {
    // T
    let a = Nested::Value(5);

    // Vec<T>
    let b = Nested::List(vec![Nested::Value(1), Nested::Value(2)]);

    // Vec<Vec<T>>
    let c = Nested::List(vec![
        Nested::List(vec![Nested::Value(1), Nested::Value(2)]),
        Nested::List(vec![Nested::Value(3)]),
    ]);

    // Vec<Vec<Vec<T>>> ... unlimited depth
    let d = Nested::List(vec![Nested::List(vec![Nested::List(vec![Nested::Value(
        42,
    )])])]);
}
