use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncResultRouter, ResultRouter};

#[test]
fn routes_ok_and_err_variants() {
    let ok_rec = Recorder::new();
    let err_rec = Recorder::new();

    let router = ResultRouter::new(ok_rec.clone(), err_rec.clone());

    router.accept(Ok::<_, &str>(1));
    router.accept(Err("oops"));

    assert_eq!(ok_rec.take(), vec![1]);
    assert_eq!(err_rec.take(), vec!["oops"]);
}

#[test]
fn routes_ok_and_err_variants_async() {
    let ok_rec = Recorder::new();
    let err_rec = Recorder::new();

    let router = AsyncResultRouter::new(ok_rec.clone(), err_rec.clone());

    block_on(router.accept_async(Ok::<_, &str>(2)));
    block_on(router.accept_async(Err("err")));

    assert_eq!(ok_rec.take(), vec![2]);
    assert_eq!(err_rec.take(), vec!["err"]);
}
