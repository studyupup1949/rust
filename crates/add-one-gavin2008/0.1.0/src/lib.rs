pub mod utils;

pub fn add_one(x:i32) ->i32{
    return x+1;
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
