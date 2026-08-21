use add_one_test_1129;
#[test]
fn add_two(){
    let q=add_one_test_1129::add_one(2);
    assert_eq!(q,3);
}