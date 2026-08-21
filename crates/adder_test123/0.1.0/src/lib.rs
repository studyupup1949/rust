//! # Adder
///Adds left ann right
/// # Examples
/// 
/// ```
/// let answer =adder:: add(1,2);
/// assert_eq!(3,answer);
/// ```
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
    #[test]
    fn exploration(){
        assert_eq!(2+2,4);
    }
    #[test]
    fn test2()->Result<(),String>{
        if add(2,2)==4{
            Ok(())
        }else{
            Err("Error".to_string())
        }
    }
}
