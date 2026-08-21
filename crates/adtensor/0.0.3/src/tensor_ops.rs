use crate::{ops::*, tensor::*};
use std::iter::{Iterator, Sum};
use std::ops::{Add, AddAssign, Mul};
use std::mem;
use typenum::{Unsigned};
use generic_array::{ArrayLength};

impl<'a, 'b, V, T, N, L> Add<&'b Tensor<V, N, T, L>> for &'a Tensor<V, N, T, L>
  where &'a V: Add<&'b V, Output=V>,
        N: ArrayLength<V> {
  type Output = Tensor<V, N, T, L>;
  fn add(self, rhs: &'b Tensor<V, N, T, L>) -> Self::Output {
    let mut tn: Self::Output = unsafe { mem::uninitialized() };  
    for (i, (a, b)) in self.iter().zip(rhs.iter()).enumerate() {
      tn[i] = a + b;
    }
    tn
  }
}

impl<'a, 'b, T, N> Dot<&'b Tensor<T, N, T, CMaj>> for &'a Tensor<T, N, T, RMaj>
  where T: Add<T, Output=T> + AddAssign<T> + Mul<T, Output=T> + Default + Sum + Sized + Copy,
        N: ArrayLength<T> + Unsigned {
  type Output = T;
  fn dot(self, rhs: &'b Tensor<T, N, T, CMaj>) -> Self::Output {
    let n = N::to_usize();
    let c = n / 8;
    let i = c * 8;
    let mut t = self[i..].into_iter()
                         .zip(rhs[i..].into_iter())
                         .map(|(&a, &b)| a + b)
                         .sum();
    if c > 0 {
      let mut p = [T::default(); 8];  
      for (a, b) in self[..].chunks(8)
                            .take(c)
                            .zip(rhs[..].chunks(8)
                                        .take(c)) {
        p[0] = a[0] * b[0];
        p[1] = a[1] * b[1];
        p[2] = a[2] * b[2];
        p[3] = a[3] * b[3];
        p[4] = a[4] * b[4];
        p[5] = a[5] * b[5];
        p[6] = a[6] * b[6];
        p[7] = a[7] * b[7];
      }
      t += p[0] + p[4];
      t += p[1] + p[5];
      t += p[2] + p[6];
      t += p[3] + p[7];
    } 
    t 
  }
}
