use adtensor::*;
use typenum::*;
use generic_array::*;

fn main() {
  let x = <tnsr![i32; U3; RMaj]>::from(
    arr![i32; 0, 1, 2]
  );
  let y = <tnsr![i32; U3; CMaj]>::from(
    arr![i32; 0, 1, 2]
  );
  let z = x.dot(&y); 
  println!("{:?} =\n{:?} dot\n{:?}", &z, &x, &y);
}
