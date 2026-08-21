pub fn add3(x: i32) -> i32 {
    x + 3
}

#[test]
#[allow(non_snake_case)]
fn it_works__0() { assert_eq!(add3( 0),  3); }
#[test]
fn it_works_14() { assert_eq!(add3(14), 17); }
#[test]
fn it_works_n1() { assert_eq!(add3(-1),  2); }
