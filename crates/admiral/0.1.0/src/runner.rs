use std::panic;

use admiral_types::RussTestDescAndFn;

// use rustc_test::TestDesc;

pub fn runner(tests: &[&RussTestDescAndFn]) {
    // dbg!(tests);
    let args = std::env::args().collect::<Vec<_>>();
    let parsed = rustc_test::test::parse_opts(&args);
    let mut opts = match parsed {
        Some(Ok(o)) => o,
        Some(Err(msg)) => panic!("{:?}", msg),
        None => return,
    };

    let mut rendered: Vec<rustc_test::TestDescAndFn> = Vec::new();
    for input in tests.iter() {
        let input = rustc_test::TestDescAndFn {
            desc: input.desc.clone(),
            testfn: rustc_test::TestFn::StaticTestFn(input.testfn),
        };
        render_test_descriptor(&input, &mut opts, &mut rendered);
    }

    match rustc_test::run_tests_console(&opts, rendered) {
        Ok(true) => {}
        Ok(false) => panic!("Some tests failed"),
        Err(e) => panic!("io error when running tests: {:?}", e),
    }
}
pub trait TestDescriptor {
    fn as_datatest_desc(&self) -> DatatestTestDesc;
}

impl TestDescriptor for rustc_test::TestDescAndFn {
    fn as_datatest_desc(&self) -> DatatestTestDesc {
        DatatestTestDesc::Test(self)
    }
}
pub enum DatatestTestDesc<'a> {
    Test(&'a rustc_test::TestDescAndFn),
}

fn render_test_descriptor(
    input: &dyn TestDescriptor,
    _opts: &mut rustc_test::TestOpts,
    rendered: &mut Vec<rustc_test::TestDescAndFn>,
) {
    match input.as_datatest_desc() {
        DatatestTestDesc::Test(test) => {
            // Make a copy as we cannot take ownership
            rendered.push(rustc_test::TestDescAndFn {
                desc: test.desc.clone(),
                testfn: clone_testfn(&test.testfn),
            });
        }
    }
}

fn clone_testfn(testfn: &rustc_test::TestFn) -> rustc_test::TestFn {
    match testfn {
        rustc_test::TestFn::StaticTestFn(func) => rustc_test::TestFn::StaticTestFn(*func),
        rustc_test::TestFn::StaticBenchFn(bench) => rustc_test::TestFn::StaticBenchFn(*bench),
        _ => unimplemented!("only static functions are supported"),
    }
}
