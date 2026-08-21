//^
//^ HEAD
//^

//> HEAD -> SUPER
use super::{
    Report,
    Same,
    Name
};

//> HEAD -> TEST
use test::Bencher;

//> HEAD -> CORE
use core::hint::black_box;


//^
//^ BENCHES
//^

//> BENCHES -> SAME
#[bench]
fn same(bencher: &mut Bencher) -> () {
    let mut report = Report::default();
    bencher.iter(|| {
        for _ in 0..10000 {
            black_box(report.to::<Same>());
        }
    });
}

//> BENCHES -> NAME
#[bench]
fn name(bencher: &mut Bencher) -> () {
    let mut report = Report::default();
    bencher.iter(|| {
        for _ in 0..10000 {
            black_box(report.to::<Name<"example">>());
        }
    });
}