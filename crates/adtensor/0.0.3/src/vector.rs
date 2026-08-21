use std::ops::{Add, AddAssign, Mul, Deref, DerefMut};
use std::iter::{FromIterator, Iterator};
use std::fmt::{Display, Debug, Formatter, Result};
use typenum::{Unsigned};
use generic_array::{GenericArray, ArrayLength};

#[derive(Debug)]
pub struct Vector<T, N>
  where N: ArrayLength<T> {
  a: GenericArray<T, N>
}

impl<T, N> Default for Vector<T, N>
  where T: Default,
        N: ArrayLength<T> {
  fn default() -> Self {
    Self{a: GenericArray::<T, N>::default()}
  }
}

impl<T, N> Display for Vector<T, N> 
  where T: Debug,
        N: ArrayLength<T> {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "{:?}", &self.a)
  }
} 

impl<T, N> Clone for Vector<T, N>
  where T: Clone,
        N: ArrayLength<T> {
  #[inline]
  fn clone(&self) -> Self {
    let mut v: Self = unsafe { std::mem::uninitialized() };
    v.deref_mut().clone_from_slice(self.deref());
    v
  }
}

impl<T, N> Copy for Vector<T, N>
  where T: Clone,
        N: ArrayLength<T>,
        GenericArray<T, N>: Copy {}
  

impl<T, N> Deref for Vector<T, N>
  where N: ArrayLength<T> + Unsigned {
  type Target = GenericArray<T, N>;
  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.a
  }
}

impl<T, N> DerefMut for Vector<T, N>
  where N: ArrayLength<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.a
  }
}

impl<T, N> FromIterator<T> for Vector<T, N>
  where N: ArrayLength<T> {
  #[inline]
  fn from_iter<I>(i: I) -> Self
    where I: IntoIterator<Item=T> {
    let mut v: Self = unsafe { std::mem::uninitialized() };
    let mut i = i.into_iter();
    for p in &mut v[..] {
      unsafe { std::ptr::write(p, i.next().unwrap()) };
    }
    v
  }
} 

impl<T, N> Vector<T, N>
  where N: ArrayLength<T> {
  pub fn dot<'a, 'b>(&'a self, v: &'b Vector<T, N>) -> T
  where T: Default + Copy + Add<T, Output=T> + AddAssign<T> + Mul<T, Output=T>,
        N: Unsigned {
    let a = self.deref().as_ptr();
    let b = v.deref().as_ptr();
    let mut s = [T::default(); 8];
    let mut i = 0;
    while i + 8 < N::to_usize() {
      s[0] = unsafe { a.add(i).read() * b.add(i).read() };
      s[1] = unsafe { a.add(i+1).read() * b.add(i+1).read() };
      s[2] = unsafe { a.add(i+2).read() * b.add(i+2).read() };
      s[3] = unsafe { a.add(i+3).read() * b.add(i+3).read() };
      s[4] = unsafe { a.add(i+4).read() * b.add(i+4).read() };
      s[5] = unsafe { a.add(i+5).read() * b.add(i+5).read() };
      s[6] = unsafe { a.add(i+6).read() * b.add(i+6).read() };
      s[7] = unsafe { a.add(i+7).read() * b.add(i+7).read() };
      i += 8;
    }
    let mut t = T::default();
    t += s[0] + s[4];
    t += s[1] + s[5];
    t += s[2] + s[6];
    t += s[3] + s[7];
    for u in i..N::to_usize() {
      t += unsafe { a.add(u).read() * b.add(u).read() };
    }
    t
  }
  pub fn map<F>(&self, f: F) -> Self 
    where T: Copy,
          F: Fn(T)->T {
    let mut v: Vector<T, N> = unsafe { std::mem::uninitialized() };
    let c = self.deref().as_ptr();
    let m = v.deref_mut().as_mut_ptr();
    for i in 0..N::to_usize() {
      unsafe { m.add(i).write(f(c.add(i).read())) };
    }
    v
  }
  pub fn apply<F>(&mut self, f: F)
    where T: Copy,
          F: Fn(T)->T {
    let c = self.deref().as_ptr();
    let m = self.deref_mut().as_mut_ptr();
    for i in 0..N::to_usize() {
      unsafe { m.add(i).write(f(c.add(i).read())) };
    }
  }
}
