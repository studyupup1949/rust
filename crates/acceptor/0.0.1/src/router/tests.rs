use core::future::Ready;

use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncRouter, AsyncRouterEntry, Router, RouterEntry};

#[test]
fn routes_to_matching_entries_in_order() {
    let even_rec = Recorder::new();
    let gt_two_rec = Recorder::new();

    fn is_even(v: &u32) -> bool {
        *v % 2 == 0
    }

    fn greater_than_two(v: &u32) -> bool {
        *v > 2
    }

    let table = Router::new(vec![
        RouterEntry::new(is_even as fn(&u32) -> bool, even_rec.clone()),
        RouterEntry::new(greater_than_two as fn(&u32) -> bool, gt_two_rec.clone()),
    ]);

    table.accept(1);
    table.accept(4);
    table.accept(2);

    assert_eq!(even_rec.take(), vec![4, 2]);
    assert_eq!(gt_two_rec.take(), vec![4]);
}

#[test]
fn routes_to_matching_entries_in_order_async() {
    let even_rec = Recorder::new();
    let gt_two_rec = Recorder::new();

    fn is_even(v: &u32) -> Ready<bool> {
        core::future::ready(*v % 2 == 0)
    }

    fn greater_than_two(v: &u32) -> Ready<bool> {
        core::future::ready(*v > 2)
    }

    let table = AsyncRouter::new(vec![
        AsyncRouterEntry::new(is_even as fn(&u32) -> Ready<bool>, even_rec.clone()),
        AsyncRouterEntry::new(
            greater_than_two as fn(&u32) -> Ready<bool>,
            gt_two_rec.clone(),
        ),
    ]);

    block_on(table.accept_async(1));
    block_on(table.accept_async(4));
    block_on(table.accept_async(2));

    assert_eq!(even_rec.take(), vec![4, 2]);
    assert_eq!(gt_two_rec.take(), vec![4]);
}
