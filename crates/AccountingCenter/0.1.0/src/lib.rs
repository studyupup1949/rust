mod mycart;
pub use mycart::*;
#[cfg(test)]
mod tests {
    use super::*;
    ///let p = Product{id:1,name:"云南青头菌".to_string(),price:100.0,num:3.0};
    ///let c = Shoppingcart::addproduct(1,"云南青头菌".to_string(),100.0,3.0);
    ///assert_eq!(p, c);
    /// dosomething
    #[test]
    fn test1() {
        let p = Product{id:1,name:"云南青头菌".to_string(),price:500.0,num:3.0};
        let c = Shoppingcart::addproduct(1,"云南青头菌".to_string(),500.0,3.0);
        assert_eq!(p, c);
    }
    #[test]
    fn test2() {
        let mut s = Shoppingcart::new();
        let p = Product{id:1,name:"云南青头菌".to_string(),price:500.0,num:3.0};
        s.items.push(p);
        assert_eq!(s.totalnum(), 1500.0);
    }
}
