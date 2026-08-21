//! Check the hard-coded GUID catalog against a real forest.
//!
//! The catalog claims a set of GUIDs are identical in every Active Directory forest. That is
//! true — but "I typed it correctly" is a separate claim, and a wrong GUID fails *silently*:
//! the ACE simply stops classifying, an edge never appears, and nothing errors. So dump the
//! real values and diff them.
//!
//! ```text
//! # on a domain-joined host, or with explicit creds — see lab_validate.ps1
//! cargo run --example verify_schema -- schema_guids.csv
//! ```
//!
//! Input is `name,guid,kind` with `kind` one of `attribute` / `right`, exactly what
//! `lab_validate.ps1` writes out of `CN=Schema` and `CN=Extended-Rights`.

use ad_acl::catalog::{self, GuidClass, KnownGuid};
use std::collections::HashMap;
use std::process::ExitCode;
use windows_sddl::sid::Guid;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: verify_schema <schema_guids.csv>");
        return ExitCode::FAILURE;
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    // name (lowercased) -> guid, as the forest reports it.
    let mut forest: HashMap<String, Guid> = HashMap::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("name,") {
            continue;
        }
        let mut f = line.split(',');
        let (Some(name), Some(guid)) = (f.next(), f.next()) else {
            eprintln!("line {}: expected name,guid[,kind]", n + 1);
            return ExitCode::FAILURE;
        };
        match Guid::parse(guid.trim()) {
            Some(g) => {
                forest.insert(name.trim().to_ascii_lowercase(), g);
            }
            None => {
                eprintln!("line {}: {guid} is not a GUID", n + 1);
                return ExitCode::FAILURE;
            }
        }
    }

    let mut wrong = 0usize;
    let mut missing = 0usize;
    let mut ok = 0usize;

    let mut check = |k: &KnownGuid| {
        let want = k.guid();
        match forest.get(&k.name.to_ascii_lowercase()) {
            Some(&got) if got == want => {
                ok += 1;
                println!("  ok       {:<45} {want}", k.name);
            }
            Some(&got) => {
                wrong += 1;
                println!("  WRONG    {:<45} catalog {want}", k.name);
                println!("           {:<45} forest  {got}", "");
            }
            None => {
                missing += 1;
                println!("  absent   {:<45} (not in the dump)", k.name);
            }
        }
    };

    println!("catalog vs forest:");
    for k in catalog::ALL {
        check(k);
    }
    // The two validated-write GUIDs alias attributes, so they are checked by name separately.
    for k in [&catalog::SELF_MEMBERSHIP, &catalog::VALIDATED_SPN] {
        if forest.contains_key(&k.name.to_ascii_lowercase()) {
            check(k);
        }
    }

    println!("\n{ok} correct · {wrong} wrong · {missing} not present in the dump");
    if wrong > 0 {
        eprintln!(
            "\nA wrong GUID is silent: the ACE stops classifying and the edge never appears. \
             Fix catalog.rs before publishing."
        );
        return ExitCode::FAILURE;
    }
    if missing > 0 {
        eprintln!(
            "\nSome entries were not in the dump — widen the filter in lab_validate.ps1 rather \
             than assuming they are fine."
        );
    }
    // Named so the compiler keeps the enum in scope for readers of this example.
    let _ = GuidClass::Attribute;
    ExitCode::SUCCESS
}
