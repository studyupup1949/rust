use criterion::*;
use adtensor::{Matrix};
use std::ops::{Add, Deref};
use std::iter::{FromIterator, Iterator, IntoIterator, repeat};
use typenum::*;
use generic_array::{GenericArray, ArrayLength};
use ndarray::{Array2};
use nalgebra as na;

fn criterion_benchmark(c: &mut Criterion) {
  type M = U64; type _M = na::U64;
  type K = U1024; type _K = U1024;
  type N = U10; type _N = na::U10;
  c.bench_function("matrix_matmul", |b| b.iter(|| {
    let x = Matrix::<f32, M, K>::from_iter(repeat(0.1));
    let w = Matrix::<f32, N, K>::from_iter(repeat(0.1));
    let y = x.matmul(&w);
    y[0]
  })); 
  c.bench_function("na_matmul", |b| b.iter(|| {
    let x = na::Matrix::<f32, _M, _K, na::ArrayStorage<f32, _M, _K>>::from_iterator(repeat(0.1));
    let w = na::Matrix::<f32, _K, _N, na::ArrayStorage<f32, _K, _N>>::from_iterator(repeat(0.1));
    let y = &x * &w;
    y[0]
  })); 
  c.bench_function("array2_matmul", |b| b.iter(|| {
    let x = Array2::<f32>::from_elem((M::to_usize(), K::to_usize()), 0.1);
    let w = Array2::<f32>::from_elem((K::to_usize(), N::to_usize()), 0.1);
    let y = &x.dot(&w);
    y[[0, 0]]
  })); 
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches); 
