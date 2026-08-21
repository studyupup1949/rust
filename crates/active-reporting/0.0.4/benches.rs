//^
//^ HEAD
//^

//> HEAD -> ACTIVE_REPORTING
use active_reporting::{
    Report,
    Same,
    Name
};

//> HEAD -> CORE
use core::hint::black_box;

//> HEAD -> CRITERION
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
    Throughput
};


//^
//^ BENCHES
//^

//> BENCHES -> SETUP
criterion_group!(active_reporting, benches);
criterion_main!(active_reporting);

//> BENCHES -> RUN
fn benches(criterion: &mut Criterion) -> () {
    let mut group = criterion.benchmark_group("active-reporting");
    group.throughput(Throughput::Elements(10000));
    let mut report = Report::default();
    group.bench_function("same", |bencher| bencher.iter(|| {for _ in 0..10000 {
        black_box(report.to::<Same>());
    }}));
    group.bench_function("name", |bencher| bencher.iter(|| {for _ in 0..10000 {
        black_box(report.to::<Name<"example">>());
    }}));
}