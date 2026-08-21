use crate::{Variety, ZAdic};


#[test]
fn roots_sorted() {

    let mut var = zadic_variety!(5, 2, [[3, 2], [3, 1], [4, 4], [0, 2], [4, 0]]);
    var.adic_sort();
    assert_eq!(zadic_variety!(5, 2, [[0, 2], [3, 1], [3, 2], [4, 0], [4, 4]]), var);
    let mut iter = var.roots();
    assert_eq!(iter.next(), Some(&zadic_approx!(5, 2, [0, 2])));
    assert_eq!(iter.next(), Some(&zadic_approx!(5, 2, [3, 1])));
    assert_eq!(iter.next(), Some(&zadic_approx!(5, 2, [3, 2])));
    assert_eq!(iter.next(), Some(&zadic_approx!(5, 2, [4, 0])));
    assert_eq!(iter.next(), Some(&zadic_approx!(5, 2, [4, 4])));
    assert_eq!(iter.next(), None);

    let var = Variety::new(vec![
        zadic_approx!(5, 3, [2, 1, 0]), zadic_approx!(5, 2, [2, 1])
    ]);
    assert_eq!(
        Variety::new(
            vec![zadic_approx!(5, 2, [2, 1]), zadic_approx!(5, 3, [2, 1, 0])]
        ),
        var.adic_sorted()
    );
    let var = Variety::new(vec![
        zadic_approx!(5, 2, [2, 1]), ZAdic::from(uadic!(5, [2, 1]))
    ]);
    assert_eq!(
        Variety::new(vec![
            ZAdic::from(uadic!(5, [2, 1])), zadic_approx!(5, 2, [2, 1])
        ]),
        var.adic_sorted()
    );

}

#[test]
fn extend_roots() {
    let mut var = zadic_variety!(5, 2, [[3, 2], [3, 1]]).adic_sorted();
    assert_eq!(
        zadic_variety!(5, 2, [[3, 1], [3, 2]]).adic_sorted(),
        var
    );
    var.extend([zadic_approx!(5, 2, [4, 4]), zadic_approx!(5, 2, [0, 2]), zadic_approx!(5, 2, [4, 0])]);
    var.adic_sort();
    assert_eq!(
        zadic_variety!(5, 2, [[0, 2], [3, 1], [3, 2], [4, 0], [4, 4]]).adic_sorted(),
        var
    );
}

#[test]
fn display() {

    let var = Variety::<ZAdic>::empty();
    assert_eq!("variety(empty)", var.to_string());

    let var = zadic_variety!(5, 6, [[0, 0, 0, 0, 0, 0], [1, 2, 3, 0, 1, 2]]);
    assert_eq!("variety(...000000._5, ...210321._5)", var.to_string());

    let var = Variety::new(vec![ZAdic::from(uadic!(5, [1, 0, 3, 0, 0, 0])), zadic_approx!(5, 6, [1, 2, 3, 0, 1, 2])]);
    assert_eq!("variety(301._5, ...210321._5)", var.to_string());

}
