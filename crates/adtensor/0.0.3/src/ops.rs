pub trait Dot<R> {
  type Output;
  fn dot(self, rhs: R) -> Self::Output;
}  
