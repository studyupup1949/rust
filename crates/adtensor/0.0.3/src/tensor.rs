use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Mul, Div};
use std::fmt::{Debug, Formatter, Result};
use std::mem;
use generic_array::{GenericArray, ArrayLength};
use typenum::{Unsigned, Prod, Quot, Same};

pub struct RMaj;
pub struct CMaj;

pub struct Tensor<V, N, T, L>
  where N: ArrayLength<V> {
  a: GenericArray<V, N>,
  p: PhantomData<(T, L)>
}

#[macro_export]
macro_rules! tnsr {
  ($t:ty; $n:ty; $l:ty) => { Tensor<$t, $n, $t, $l> };
  ($t:ty; $n1:ty $(, $ntail:ty)*; RMaj) => { Tensor<tnsr![$t; $($ntail), *; RMaj], $n1, $t, RMaj> };
  ($t:ty; $n2:ty, $n1:ty; CMaj) => { Tensor<tnsr![$t; $n2; CMaj], $n1, $t, CMaj> };
  ($t:ty; $n2:ty, $n1:ty $(, $ntail:ty)*; CMaj) => { Tensor<tnsr![$t; $n1, $($ntail), *; CMaj], $n2, $t, CMaj> } 
} 

impl<V, N, T, L> Default for Tensor<V, N, T, L>
  where N: ArrayLength<V>,
        V: Default {
  fn default() -> Self {
    Self{a: GenericArray::<V, N>::default(), p: PhantomData::<(T, L)>::default()}
  }
}

impl<V, N, T, L> Clone for Tensor<V, N, T, L>
  where N: ArrayLength<V>,
        GenericArray<V, N>: Clone {
  fn clone(&self) -> Self {
    Self{a: self.a.clone(), p: PhantomData::<(T, L)>::default()}
  }
}

impl<V, N, T, L> Copy for Tensor<V, N, T, L> 
  where N: ArrayLength<V>,
        GenericArray<V, N>: Copy {}

impl<V, N, T, L> Deref for Tensor<V, N, T, L>
  where N: ArrayLength<V>,
        GenericArray<V, N>: Deref {
   type Target = [V];
  fn deref(&self) -> &Self::Target {
    &self.a.as_slice()
  }
}

impl<V, N, T, L> DerefMut for Tensor<V, N, T, L>
  where N: ArrayLength<V>,
        GenericArray<V, N>: DerefMut { 
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.a.as_mut_slice()
  }
} 

impl<T, N> Debug for Tensor<T, N, T, RMaj>
  where N: ArrayLength<T>,
        T: Debug {
  fn fmt(&self, f: &mut Formatter) -> Result {
    let mut s = String::default();
    for (j, t) in self.iter().enumerate() {
      if j > 0 {
        s += &String::from(" ");
      }
      s += &format!("{:?}", t);
    }
    write!(f, "[{}]", s)
  }
}

impl<T, N> Debug for Tensor<T, N, T, CMaj>
  where N: ArrayLength<T>,
        T: Debug {
  fn fmt(&self, f: &mut Formatter) -> Result {
    let mut s = String::default();
    for (j, t) in self.iter().enumerate() {
      if j > 0 {
        s += &String::from("\n ");
      }
      s += &format!("{:?}", t);
    }
    write!(f, "[{}]", s)
  }
}

impl<M, N, T> Debug for Tensor<Tensor<T, N, T, RMaj>, M, T, RMaj>
  where M: ArrayLength<Tensor<T, N, T, RMaj>>,
        N: ArrayLength<T>,
        T: Debug {
  fn fmt(&self, f: &mut Formatter) -> Result {
    let mut s = String::default();
    for (i, r) in self.iter().enumerate() {
      if i > 0 {
        s += &String::from("\n ");
      }
      for (j, t) in r.iter().enumerate() {
        if j > 0 {
          s += &String::from(" ");
        }
        s += &format!("{:?}", t);
      }
    }
    write!(f, "[{}]", s)
  }
} 

impl<M, N, T> Debug for Tensor<Tensor<T, N, T, CMaj>, M, T, CMaj>
  where M: ArrayLength<Tensor<T, N, T, CMaj>> + Unsigned,
        N: ArrayLength<T> + Unsigned,
        T: Debug {
  fn fmt(&self, f: &mut Formatter) -> Result {
    let mut s = String::default();
    for i in 0..N::to_usize() {
      if i > 0 {
        s += &String::from("\n ");
      }
      for j in 0..M::to_usize() {
        if j > 0 {
          s += &String::from(" ");
        }
        s += &format!("{:?}", self[j][i]);
      }
    }
    write!(f, "[{}]", s)
  }
} 

impl<V, K, M, N, T, L> Debug for Tensor<Tensor<Tensor<V, N, T, L>, M, T, L>, K, T, L>
  where K: ArrayLength<Tensor<Tensor<V, N, T, L>, M, T, L>>,
        M: ArrayLength<Tensor<V, N, T, L>>,
        N: ArrayLength<V>,
        Tensor<Tensor<V, N, T, L>, M, T, L>: Debug {
  fn fmt(&self, f: &mut Formatter) -> Result {
    let mut s = String::default();
    for (i, r) in self.iter().enumerate() {
      if i > 0 {
        s += &String::from("\n ");
      }
      s += &format!("{:?}", r).replace("\n", "\n ");
    }
    write!(f, "[{}]", s)
  }
} 

pub trait NItems {
  fn nitems() -> usize;
}

impl<T, N, L> NItems for Tensor<T, N, T, L>
  where N: ArrayLength<T> + Unsigned {
  fn nitems() -> usize { N::to_usize() }
}

impl<V, M, N, T, L> NItems for Tensor<Tensor<V, N, T, L>, M, T, L>
  where N: ArrayLength<V>,
        Tensor<V, N, T, L>: NItems,
        M: ArrayLength<Tensor<V, N, T, L>> + Unsigned {
  fn nitems() -> usize { M::to_usize() * Tensor::<V, N, T, L>::nitems() }
}


impl<T, N, L> From<GenericArray<T, N>> for Tensor<T, N, T, L>
  where N: ArrayLength<T> {
  fn from(a: GenericArray<T, N>) -> Self {
    Self{a, p: PhantomData::<(T, L)>::default()}
  }
}

impl<T, M, N, S> From<GenericArray<T, S>> for Tensor<Tensor<T, N, T, RMaj>, M, T, RMaj>
  where M: ArrayLength<Tensor<T, N, T, RMaj>> + Mul<N>,
        N: ArrayLength<T> + Unsigned,
        S: ArrayLength<T> + Same<Prod<M, N>>,
        T: Copy {
  fn from(a: GenericArray<T, S>) -> Self {
    let mut tn =  Self{a: unsafe { mem::uninitialized() }, 
                       p: PhantomData::<(T, RMaj)>::default()};
    let mut s = &a[..];
    let n = N::to_usize();
    for r in tn.iter_mut() {
       r[..].copy_from_slice(&s[..n]);
       s = &s[n..];
    }
    tn
  }
}

impl<T, M, N, S> From<GenericArray<T, S>> for Tensor<Tensor<T, M, T, CMaj>, N, T, CMaj>
  where M: ArrayLength<T> + Mul<N> + Unsigned,
        N: ArrayLength<Tensor<T, M, T, CMaj>> + Unsigned,
        S: ArrayLength<T> + Same<Prod<M, N>>,
        T: Copy {
  fn from(a: GenericArray<T, S>) -> Self {
    let mut tn =  Self{a: unsafe { mem::uninitialized() }, 
                       p: PhantomData::<(T, CMaj)>::default()};
    let n = N::to_usize();
    let m = M::to_usize();
    let mut u = 0;
    for j in 0..n {
      for i in 0..m {
        tn[j][i] = a[u];
        u += 1;
      }
    }
    tn
  }
}

impl<V, T, K, M, N, L, S> From<GenericArray<T, S>> for Tensor<Tensor<Tensor<V, N, T, L>, M, T, L>, K, T, L>
  where K: ArrayLength<Tensor<Tensor<V, N, T, L>, M, T, L>> + Unsigned, 
        M: ArrayLength<Tensor<V, N, T, L>>,
        N: ArrayLength<V>,
        S: ArrayLength<T> + Div<K>,
        Quot<S, K>: ArrayLength<T>,
        Tensor<Tensor<V, N, T, L>, M, T, L>: From<GenericArray<T, Quot<S, K>>>,
        Tensor<V, N, T, L>: NItems,
        T: Copy,
        GenericArray<T, Quot<S, K>>: Copy {
  fn from(a: GenericArray<T, S>) -> Self {
    let mut tn: Self =  unsafe { mem::uninitialized() };
    let mut s = &a[..];
    let mut ra: GenericArray<T, Quot<S, K>> = unsafe { mem::uninitialized() };
    let q = Tensor::<Tensor<V, N, T, L>, M, T, L>::nitems();
    let k = K::to_usize();
    for i in 0..k {
      ra[..].copy_from_slice(&s[..q]);
      tn[i] = Tensor::<Tensor<V, N, T, L>, M, T, L>::from(ra);
      s = &s[q..];
    }
    tn
  }
}
