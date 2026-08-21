use crate::vector::Vector;
use std::ops::{Deref, DerefMut, Add, AddAssign, Mul};
use std::iter::{FromIterator};
use std::fmt::{Display, Debug, Formatter, Result};
use generic_array::{GenericArray, ArrayLength};
use typenum::{Unsigned};

#[derive(Debug)]
pub struct Matrix<T, R, C>
  where C: ArrayLength<T>,
        R: ArrayLength<Vector<T, C>> {
  a: GenericArray<Vector<T, C>, R>
}

impl<T, R, C> Display for Matrix<T, R, C> 
  where T: Debug,
        C: ArrayLength<T>,
        R: ArrayLength<Vector<T, C>> {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f , "[{}]", self.a.iter()
                             .map(|v| format!("{}", v))
                             .collect::<Vec<String>>()
                             .join(", "))
  }
} 

impl<T, R, C> Deref for Matrix<T, R, C>
  where C: ArrayLength<T>,
        R: ArrayLength<Vector<T, C>> {
  type Target = GenericArray<Vector<T, C>, R>;
  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.a
  }
}

impl<T, R, C> DerefMut for Matrix<T, R, C>
  where C: ArrayLength<T>,
        R: ArrayLength<Vector<T, C>>  {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.a
  }
}

impl<T, R, C> FromIterator<T> for Matrix<T, R, C>
  where C: ArrayLength<T>,
        R: ArrayLength<Vector<T, C>> {
  fn from_iter<I>(i: I) -> Self
    where I: IntoIterator<Item=T> {
    let mut m: Self = unsafe { std::mem::uninitialized() };
    let mut i = i.into_iter();
    for v in &mut m[..] {
      for p in &mut v[..] {
        unsafe { std::ptr::write(p, i.next().unwrap()) };
      }
    }
    m
  }
}

impl<T, M, K> Matrix<T, M, K>
  where K: ArrayLength<T> + Unsigned,
        M: ArrayLength<Vector<T, K>> + Unsigned {
  pub fn matmul<'b, N>(&self, cmat: &'b Matrix<T, N, K>) -> Matrix<T, M, N>
    where T: Default + Copy + Add<T, Output=T> + AddAssign<T> + Mul<T, Output=T>,
          N: ArrayLength<Vector<T, K>> + ArrayLength<T> + Unsigned,
          M: ArrayLength<Vector<T, N>> {
    let mut out: Matrix<T, M, N> = unsafe { std::mem::uninitialized() };
    for (r, row) in self.iter().enumerate() {
      for (c, col) in cmat.iter().enumerate() {
        out[r][c] = row.dot(&col);
      }
    }
    out
  }
}
