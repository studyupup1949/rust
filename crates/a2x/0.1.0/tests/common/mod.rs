use a2x::alfa_compile;
use a2x::context::Context;
use a2x::xacml::xpolicy::XPolicy;
use a2x::xacml::xpolicyset::XPolicySet;
use a2x::xacml::{XTopPolicy, XacmlWriter};
use a2x::AlfaFile;
use std::path::PathBuf;
use std::rc::Rc;
use unwrap::unwrap;
use xml::EmitterConfig;

/// Compile a single ALFA source text.
/// Panics on compilation failure.
pub fn compile_alfa_src(src: &str) -> Vec<XTopPolicy> {
    // default context.
    let ctx = Rc::new(Context::default());
    // a single file.
    let sources = vec![AlfaFile {
        filename: PathBuf::default(),
        contents: src.to_owned(),
    }];
    unwrap!(alfa_compile(&ctx, sources), "compile failed")
}

/// Compile multiple ALFA source texts from strings
///
/// Panics on compilation failure.
#[allow(dead_code)]
pub fn compile_alfa_srcs(src: Vec<String>) -> Vec<XTopPolicy> {
    let base_name = "alfa_";
    // name the files
    let alfa_sources = src
        .into_iter()
        .enumerate()
        .map(|(index, s)| AlfaFile {
            filename: PathBuf::from(format!("{base_name}{index}")),
            contents: s,
        })
        .collect();
    // default context.
    let ctx = Rc::new(Context::default());
    unwrap!(alfa_compile(&ctx, alfa_sources), "compile failed")
}

/// Get the Nth (0-indexed) Top-Level Policy (or panic)
#[allow(dead_code)]
pub fn get_nth_policy(n: usize, policies: Vec<XTopPolicy>) -> XPolicy {
    assert!(policies.len() > n, "There were not {n} policies");
    match policies.into_iter().nth(n).unwrap() {
        XTopPolicy::Policy(p) => p,
        XTopPolicy::PolicySet(_) => panic!("Expected Policy"),
    }
}

/// Get the Nth (0-indexed) Top-Level `PolicySet` (or panic)
#[allow(dead_code)]
pub fn get_nth_policyset(n: usize, policies: Vec<XTopPolicy>) -> XPolicySet {
    assert!(policies.len() > n, "There were not {n} policies");
    match policies.into_iter().nth(n).unwrap() {
        XTopPolicy::PolicySet(ps) => ps,
        XTopPolicy::Policy(_) => panic!("Expected PolicySet"),
    }
}

/// Convert `XTopPolicy` to a (XACML) string
#[allow(dead_code)]
pub fn xentry_to_str(p: &XTopPolicy) -> String {
    let mut target = vec![];
    // ensure directory exists
    match p {
        XTopPolicy::PolicySet(xps) => {
            let mut writer = EmitterConfig::new()
                .perform_indent(true)
                .create_writer(&mut target);
            xps.write_xml(&mut writer).expect("unable to write");
        }
        XTopPolicy::Policy(xp) => {
            let mut writer = EmitterConfig::new()
                .perform_indent(true)
                .create_writer(&mut target);
            xp.write_xml(&mut writer).expect("unable to write");
        }
    }
    // convert to a string
    String::from_utf8(target).expect("can't convert to utf8")
}
