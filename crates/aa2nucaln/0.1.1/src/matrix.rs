use crate::codons;
use anyhow::{ensure, Context, Result};
// https://stackoverflow.com/questions/64498617/how-to-transpose-a-vector-of-vectors-in-rust

pub fn transpose<T>(mut v: Vec<Vec<T>>) -> Result<Vec<Vec<T>>> {
    ensure!(!v.is_empty(), "Nested Vec<T> is empty");

    for inner in &mut v {
        inner.reverse();
    }

    Ok((0..v[0].len())
        .map(|_| {
            v.iter_mut()
                .map(|inner| inner.pop())
                .collect::<Option<Vec<T>>>()
        })
        .collect::<Option<Vec<Vec<T>>>>()
        .context("Could not transpose nested Vec<T>.")?)
}

pub fn create_matrix(aa: &[u8], ignore: bool) -> Vec<Vec<&str>> {
    let mut vec = Vec::new();
    for a in aa {
        match ignore {
            true => vec.push(codons::get_codon_string(*a)),
            false => vec.push(codons::get_codon_string_both(*a)),
        }
    }
    vec
}
