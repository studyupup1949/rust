use criterion::*;
use adtensor::{Vector};
use std::ops::{Add, AddAssign, Mul, Deref};
use std::mem;
use std::iter::{FromIterator, Iterator, IntoIterator, repeat};
use typenum::*;
use generic_array::{GenericArray, ArrayLength};
use ndarray::{Array1};
use nalgebra as na;

fn vec_zeros<T, N>() -> Vec<T>
  where T: Default + Clone,
        N: Unsigned {
  let mut v = Vec::<T>::default();
  v.resize(<N as Unsigned>::to_usize(), T::default());
  v
}

fn array1_zeros<T, N>() -> Array1<T>
  where T: Default + Clone,
        N: Unsigned {
  Array1::<T>::from_vec(vec_zeros::<T, N>())
}

struct Raw<T> {
  t: T
}

impl<T> Default for Raw<T> 
  where T: Default {
  fn default() -> Self {
    Self{t: T::default()}
  }
}

impl<T> Clone for Raw<T>
  where T: Clone {
  fn clone(&self) -> Self {
    Self{t: self.t.clone()}
  }
}

impl<T> Copy for Raw<T> where T: Copy {}

impl<T> Add<Raw<T>> for Raw<T>
  where T: Add<T, Output=T> {
  type Output = Raw<T>;
  fn add(self, rhs: Self) -> Self {
    Self{t: self.t + rhs.t}
  }
}

impl<T> AddAssign<Raw<T>> for Raw<T>
  where T: AddAssign<T> {
  fn add_assign(&mut self, rhs: Self) {
    self.t += rhs.t;
  }
}

impl<T> Mul<Raw<T>> for Raw<T>
  where T: Mul<T, Output=T> {
  type Output = Raw<T>;
  fn mul(self, rhs: Self) -> Self {
    Self{t: self.t * rhs.t}
  }
}

 
fn criterion_benchmark(c: &mut Criterion) {
  type T = f32;
  type N = U100;
  type M = na::U100;
  c.bench_function("vector_raw_dot", |b| b.iter(|| {
    let x = Vector::<Raw<T>, N>::default();
    let y = Vector::<Raw<T>, N>::default();
    x.dot(&y)
  }));
  c.bench_function("vector_dot", |b| b.iter(|| {
    let x = Vector::<T, N>::default();
    let y = Vector::<T, N>::default();
    x.dot(&y)
  }));
  /*c.bench_function("na_arr_dot", |b| b.iter(|| {
    let x = na::Matrix::<T, M, na::U1, na::ArrayStorage<T, M, na::U1>>::from_iterator(repeat(0.1));
    let y = na::Matrix::<T, M, na::U1, na::ArrayStorage<T, M, na::U1>>::from_iterator(repeat(0.1));
    x.dot(&y)
  }));
  c.bench_function("array1_dot", |b| b.iter(|| {
    let x = Array1::from_iter(repeat(0.1).take(N::to_usize()));
    let y = Array1::from_iter(repeat(0.1).take(N::to_usize()));
    x.dot(&y)
  }));
  c.bench_function("vector_map", |b| b.iter(|| {
    let x = Vector::<T, N>::from_iter(repeat(0.1));
    let w = x.map(|t| (t+1.0)/(t-1.0));
    w[0]
  }));
  c.bench_function("vector_apply", |b| b.iter(|| {
    let mut x = Vector::<T, N>::from_iter(repeat(0.1));
    x.apply(|t| (t+1.0)/(t-1.0));
    x[0]
  }));
  c.bench_function("na_apply", |b| b.iter(|| {
    let mut x = na::Matrix::<T, M, na::U1, na::ArrayStorage<T, M, na::U1>>::from_iterator(repeat(0.1));
    x.apply(|t| (t+1.0)/(t-1.0));
    x[0]
  }));
  c.bench_function("array1_apply", |b| b.iter(|| {
    let mut x = Array1::from_iter(repeat(0.1).take(N::to_usize()));
    for i in 0..x.len() {
      x[i] = (x[i]+1.0)/(x[i]-1.0);
    }
    x[0]
  }));*/
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches); 
