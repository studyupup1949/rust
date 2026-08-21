use accio::*;

/*
this example demonstrates populating a struct's
fields from different places in the code.
*/

fn main() {
    println!(
        "{:?}",
        CmdArgs::parse(&["test", "-flag1", "test_val", "-flag2",])
    );
}

#[accio_body(fields)]
#[derive(Debug, Default)]
pub struct CmdArgs {
    // fields are populated here
}

impl CmdArgs {
    /*
    for non-exhaustive match statements of this kind;
    we MUST have a final "_ => {}," block. since accio
    does not guarantee order within a given scope; we
    should use multiple scopes delimited with +; which
    are ordered within themselves.
    here, the lastCheck scope contains only one item that
    is guaranteed to come after everything in flagChecks.
    */
    #[accio_body(flagChecks + lastCheck)]
    fn parse(args: &[&str]) -> Self {
        let mut c = Self::default();
        let mut it = args.iter().skip(1);
        while let Some(f) = it.next() {
            if let Some(k) = f.strip_prefix("-") {
                match k {}
            }
        }
        c
    }
}

accio_emit! {
    lastCheck{
        // insert the fallback case here
        _ => {}
    }
}

accio_emit! {
    fields {
        pub flag1: Option<String>,
    }
    flagChecks{
        "flag1" => {
            if let Some(v) = it.next() {
                c.flag1 = Some(v.to_string());
            }
        },
    }
}

accio_emit! {
    fields {
        pub flag2: bool,
    }
    flagChecks{
        "flag2" => {
            c.flag2 = true;
        },
    }
}
