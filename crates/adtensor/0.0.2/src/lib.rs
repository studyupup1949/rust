use std::ops::{Add, Sub, Mul, Div};
use std::iter::{IntoIterator, Iterator, repeat};
use std::fmt::{Debug, Formatter, Result};
//use std::marker::{PhantomData};
use typenum::{Unsigned, /*U0, TypeArray,*/ TArr, ATerm, Prod, Max};
use generic_array::{GenericArray, ArrayLength, functional::FunctionalSequence};

type Maxed<A, B> = <A as Max<B>>::Output;

pub trait ReduceMul {
  type Output;
  fn reducemul(&self) -> Self::Output;
}

pub type ReduceProd<A> = <A as ReduceMul>::Output;

impl<V> ReduceMul for TArr<V, ATerm> {
  type Output = V;
  fn reducemul(&self) -> Self::Output {
    unsafe { ::core::mem::uninitialized() }
  }
}

impl<V, A> ReduceMul for TArr<V, A>
  where A: ReduceMul,
        V: Mul<ReduceProd<A>> {
  type Output = Prod<V, ReduceProd<A>>;
  fn reducemul(&self) -> Self::Output {
    unsafe { ::core::mem::uninitialized() }
  }
}

/*pub trait TCats<A> {
  type Output;
}

pub type TCat<A, B> = <A as TCats<B>>::Output;

impl<A> TCats<A> for ATerm
  where A: TypeArray {
  type Output = A;
}

impl<V, A> TCats<ATerm> for TArr<V, A> {
  type Output = Self;
}

impl<V, W, A, B> TCats<TArr<W, B>> for TArr<V, A>
  where TArr<W, TArr<V, A>>: TCats<B> {
  type Output = TCat<TArr<W, TArr<V, A>>, B>;
}*/

pub trait UnsignedName {
  fn name() -> String;  
}

impl<U> UnsignedName for U
  where U: Unsigned {
  fn name() -> String {
    format!("U{}", U::to_usize())
  }
}

pub trait ShapeName {
  fn name() -> String;
}

impl<V> ShapeName for TArr<V, ATerm>
  where V: UnsignedName {
  fn name() -> String { V::name() }
}

impl<V, A> ShapeName for TArr<V, A>
  where V: UnsignedName,
        A: ShapeName {
  fn name() -> String { 
    format!("{}, {}", V::name(), A::name()) 
  }
} 

pub trait One {
  fn one() -> Self;
}

impl One for f32 {
  fn one() -> Self { 1.0 }
}

impl One for f64 {
  fn one() -> Self { 1.0 }
}

pub struct ADScalar<T, P>
  where P: ArrayLength<T> {
  pub v: T,
  pub g: GenericArray<T, P>
}

pub fn adscalar<T, P>(v: T, g: GenericArray<T, P>) -> ADScalar<T, P>
  where P: ArrayLength<T> {
  ADScalar::<T, P>{v, g}
}

impl<T, P> Default for ADScalar<T, P>
  where P: ArrayLength<T>,
        T: Default {
  fn default() -> Self {
    adscalar(T::default(), GenericArray::<T, P>::default())
  }
}

impl<T, P> Clone for ADScalar<T, P>
  where P: ArrayLength<T>,
        T: Clone {
  fn clone(&self) -> Self {
    adscalar(self.v.clone(), self.g.clone())
  }
}

impl<T, P> Copy for ADScalar<T, P> 
  where P: ArrayLength<T>, T: Copy, GenericArray<T, P>: Copy {}

impl<T, P> Debug for ADScalar<T, P> where P: ArrayLength<T>, T: Debug {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "({:?}, {:?})", &self.v, &self.g)
  }
} 

pub trait FloatMath {
  fn exp(self) -> Self;
  fn ln(self) -> Self;  
}

impl FloatMath for f32 {
  fn exp(self) -> Self {
    f32::exp(self)
  }
  fn ln(self) -> Self {
    f32::ln(self)
  }
}

impl FloatMath for f64 {
  fn exp(self) -> Self {
    f64::exp(self)
  }
  fn ln(self) -> Self {
    f64::ln(self)
  }
}


impl<T, P> Add<T> for ADScalar<T, P>
  where P: ArrayLength<T>,
        T: Add<T, Output=T> {
  type Output = Self;
  fn add(self, rhs: T) -> Self {
    adscalar(self.v + rhs, self.g)
  }
} 



impl<T, P> Sub<T> for ADScalar<T, P>
  where P: ArrayLength<T>,
        T: Sub<T, Output=T> {
  type Output = Self;
  fn sub(self, rhs: T) -> Self {
    adscalar(self.v - rhs, self.g)
  }
} 

impl<T, P> Mul<T> for ADScalar<T, P>
  where P: ArrayLength<T>,
        T: Mul<T, Output=T> + Clone {
  type Output = Self;
  fn mul(self, rhs: T) -> Self {
    adscalar(self.v * rhs.clone(), self.g.map(|d| d * rhs.clone()))
  }
} 

impl<T, P> Div<T> for ADScalar<T, P>
  where P: ArrayLength<T>,
        T: Div<T, Output=T> + Clone {
  type Output = Self;
  fn div(self, rhs: T) -> Self {
    adscalar(self.v / rhs.clone(), self.g.map(|d| d / rhs.clone()))
  }
}

impl<T, P1> ADScalar<T, P1> 
  where P1: Unsigned + ArrayLength<T> {
  pub fn zip<P2, F>(self, rhs: ADScalar<T, P2>, f: F) -> ADScalar<T, Maxed<P1, P2>> 
    where P1: Max<P2>,
          P2: Unsigned + ArrayLength<T>,
          Maxed<P1, P2>: ArrayLength<T>,
          ADScalar<T, Maxed<P1, P2>>: Default,
          F: Fn(T, T) -> T,
          T: Default + Clone {
    let p1 = <P1 as Unsigned>::to_usize();
    let p2 = <P2 as Unsigned>::to_usize();
    let v = f(self.v, rhs.v);
    if p1 == p2 {
      return adscalar(v, self.g.into_iter()
                               .zip(rhs.g.into_iter())
                               .map(|(d1, d2)| f(d1, d2))
                               .collect());
    }
    else if p1 < p2 {
      return adscalar(v, self.g.into_iter()
                               .chain(repeat(T::default()))
                               .zip(rhs.g.into_iter())
                               .map(|(d1, d2)| f(d1, d2))
                               .collect());
    }
    else {
      return adscalar(v, rhs.g.into_iter()
                              .chain(repeat(T::default()))
                              .zip(self.g.into_iter())
                              .map(|(d1, d2)| f(d1, d2))
                              .collect());
    }
  }
}

impl<T, P1, P2> Add<ADScalar<T, P2>> for ADScalar<T, P1>
  where P1: Unsigned + Max<P2> + ArrayLength<T>,
        P2: Unsigned + ArrayLength<T>,
        Maxed<P1, P2>: ArrayLength<T>,
        ADScalar<T, Maxed<P1, P2>>: Default,
        T: Add<T, Output=T> + Default + Clone {
  type Output = ADScalar<T, Maxed<P1, P2>>;
  fn add(self, rhs: ADScalar<T, P2>) -> Self::Output {
    self.zip(rhs, |d1, d2| d1 + d2)
  }
}

impl<T, P1, P2> Sub<ADScalar<T, P2>> for ADScalar<T, P1>
  where P1: Unsigned + Max<P2> + ArrayLength<T>,
        P2: Unsigned + ArrayLength<T>,
        Maxed<P1, P2>: ArrayLength<T>,
        ADScalar<T, Maxed<P1, P2>>: Default,
        T: Sub<T, Output=T> + Default + Clone {
  type Output = ADScalar<T, Maxed<P1, P2>>;
  fn sub(self, rhs: ADScalar<T, P2>) -> Self::Output {
    self.zip(rhs, |d1, d2| d1 - d2)
  }
}

impl<T, P1, P2> Mul<ADScalar<T, P2>> for ADScalar<T, P1>
  where P1: Unsigned + Max<P2> + ArrayLength<T>,
        P2: Unsigned + ArrayLength<T>,
        Maxed<P1, P2>: ArrayLength<T>,
        ADScalar<T, Maxed<P1, P2>>: Default,
        T: Add<T, Output=T> + Mul<T, Output=T> + Default + Clone {
  type Output = ADScalar<T, Maxed<P1, P2>>;
  fn mul(self, rhs: ADScalar<T, P2>) -> Self::Output {
    let v1 = self.v.clone();
    let v2 = rhs.v.clone();
    self.zip(rhs, |d1, d2| d1 * v2.clone() + v1.clone() * d2)
  }
}

impl<T, P1, P2> Div<ADScalar<T, P2>> for ADScalar<T, P1>
  where P1: Unsigned + Max<P2> + ArrayLength<T>,
        P2: Unsigned + ArrayLength<T>,
        Maxed<P1, P2>: ArrayLength<T>,
        ADScalar<T, Maxed<P1, P2>>: Default,
        T: Sub<T, Output=T> + Div<T, Output=T> + Mul<T, Output=T> + Default + Clone + One {
  type Output = ADScalar<T, Maxed<P1, P2>>;
  fn div(self, rhs: ADScalar<T, P2>) -> Self::Output {
    let v1 = self.v.clone();
    let v2 = rhs.v.clone();
    let s = T::one() / (v2.clone() * v2.clone());
    self.zip(rhs, |d1, d2| (d1 * v2.clone() - v1.clone() * d2) * s.clone())
  }
}

/*impl<T, P> ADScalar<T, P>
  where T: FloatMath {
  pub fn exp(self) -> Self
    where T: Mul<T, Output=T> + Clone {
    let v = self.v.clone().exp();
    adscalar(v, self.d * self.v.exp())
  }
  pub fn ln(self) -> Self
    where T: Div<T, Output=T> + Copy {
    adscalar(self.v.ln(), self.d / self.v)
  }
}

trait ADName {
  fn name() -> String;
}

pub struct AD<P> {
  p: PhantomData<P>
}

impl<P, U> Mul<U> for AD<P>
  where P: Mul<U> {
  type Output = Prod<P, U>;
  fn mul(self, rhs: U) -> Self::Output {
    unsafe { ::core::mem::uninitialized() }
  }
} 

impl<P> ADName for AD<P>
  where P: UnsignedName {
  fn name() -> String {
    P::name()
  }
}
*/
/*pub struct NoAD {}

impl<U> Mul<U> for NoAD {
  type Output = U0;
  fn mul(self, rhs: U) -> Self::Output {
    unsafe { ::core::mem::uninitialized() }
  } 
}

impl ADName for NoAD {
  fn name() -> String {
    String::from("NoAD")
  }
}*/
/*
pub struct ADTensor<T, S, P>
  where S: ReduceMul,
        ReduceProd<S>: ArrayLength<T>,
        P: Mul<ReduceProd<S>>,
        Prod<P, ReduceProd<S>>: ArrayLength<T> {
  v: GenericArray<T, ReduceProd<S>>,
  g: GenericArray<T, Prod<P, ReduceProd<S>>>
}

pub fn adtensor<T, S, P>(v: GenericArray<T, ReduceProd<S>>, g: GenericArray<T, Prod<P, ReduceProd<S>>>) -> ADTensor<T, S, P>
  where S: ReduceMul,
        ReduceProd<S>: ArrayLength<T>,
        P: Mul<ReduceProd<S>>,
        Prod<P, ReduceProd<S>>: ArrayLength<T> {
  ADTensor::<T, S, P>{v, g}
}

impl<T, S, P> Default for ADTensor<T, S, P>
  where S: ReduceMul,
        ReduceProd<S>: ArrayLength<T>,
        P: Mul<ReduceProd<S>>,
        Prod<P, ReduceProd<S>>: ArrayLength<T>,
        T: Default {
  fn default() -> Self {
    adtensor(GenericArray::<T, ReduceProd<S>>::default(), GenericArray::<T, Prod<P, ReduceProd<S>>>::default())
  }
}

impl<T, S, P> Clone for ADTensor<T, S, P> 
  where S: ReduceMul,
        ReduceProd<S>: ArrayLength<T>,
        P: Mul<ReduceProd<S>>,
        Prod<P, ReduceProd<S>>: ArrayLength<T>,
        T: Clone {
  fn clone(&self) -> Self {
    adtensor(self.v.clone(), self.g.clone())
  }
} 

impl<T, S, P> Copy for ADTensor<T, S, P> 
  where S: ReduceMul,
        ReduceProd<S>: ArrayLength<T>,
        P: Mul<ReduceProd<S>>,
        Prod<P, ReduceProd<S>>: ArrayLength<T>,
        T: Copy,
        GenericArray<T, ReduceProd<S>>: Copy,
        GenericArray<T, Prod<P, ReduceProd<S>>>: Copy {
}

impl<T, S, P> Debug for ADTensor<T, S, P> 
  where S: ReduceMul + ShapeName,
        ReduceProd<S>: ArrayLength<T>,
        P: Mul<ReduceProd<S>> + ADName,
        Prod<P, ReduceProd<S>>: ArrayLength<T>,
        T: Debug {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "(<{}>{:?}, {}{:?})", <S as ShapeName>::name(), &self.v, <P as ADName>::name(), &self.g)
  }
} 

impl<T, S> IntoIterator for ADTensor<T, S, NoAD>
  where S: ReduceMul,
        ReduceProd<S>: ArrayLength<T> {
  type Item = T;
  type IntoIter = GenericArrayIter<T, ReduceProd<S>>;
  fn into_iter(self) -> Self::IntoIter {
    self.v.into_iter()
  }
}

impl<T, S, P> IntoIterator for ADTensor<T, S, AD<P>>
  where S: ReduceMul,
        ReduceProd<S>: ArrayLength<T>,
        P: Mul<ReduceProd<S>>,
        Prod<P, ReduceProd<S>>: ArrayLength<T>,
        P: Unsigned {
  type Item = T;
  type IntoIter = Map<Zip<GenericArrayIter<T, ReduceProd<S>>, Cycle<GenericArrayIter<T, Prod<P, ReduceProd<S>>>>>;
  fn into_iter(self) -> Self::IntoIter {
    self.v.into_iter()
  }
}
*/




