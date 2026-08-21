use accio::*;

/*
this example demonstrates populating an
enum's body and a match statement. each of
the enum variants come from a different path.
*/

fn main() {
    // should print:
    //  val1: 'one'
    //  val2: #2
    for s in vec!["s_one", "2", "none"] {
        if let Some(v) = TestEnum::parse(s) {
            v.show();
        }
    }
}

#[accio_body(testEnum)]
enum TestEnum {
    // we can't use accio::accio!(testEnum); here:
    // parser wouldn't accept macro in enum body.
    // accio_body exists as a hack for this case.
    // this block is the first (only) empty group,
    // so it will be filled by the macro.
}

impl TestEnum {
    fn parse(s: &str) -> Option<Self> {
        // inline expansion
        accio!(testParse);
        None
    }

    #[accio_body(testMatch)]
    fn show(&self) {
        match self {
            /* this block is the first empty group,
               so it will be filled by the macro */
        }
    }
}
