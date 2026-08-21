use proc_macro2::{
    Delimiter::{Brace, Bracket},
    Group, TokenStream as TS, TokenTree as TT,
};
use std::collections::HashSet;
use std::error::Error;
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream as PS};
use syn::{fold::Fold, Ident, LitStr, Macro, Token};

// name of the emitter macro
static EMIT: &str = "accio_emit";

// arguments for the collector macro
pub(crate) struct Args {
    scopes: Vec<String>,
    paths: Vec<String>,
}

impl Parse for Args {
    // args => scope_ident (+ scope_ident)* (, "path_literal")*
    fn parse(t: PS) -> syn::Result<Self> {
        // scope defines emitted code blocks to match
        let mut scopes = vec![t.parse::<Ident>()?.to_string()];
        while !t.is_empty() {
            if t.parse::<Token![+]>().is_err() {
                break;
            }
            scopes.push(t.parse::<Ident>()?.to_string());
        }

        // if optionally other paths are
        // included; add them to the scope
        let mut paths = vec![];
        while !t.is_empty() {
            t.parse::<Token![,]>()?;
            paths.push(t.parse::<LitStr>()?.value());
        }
        // caller path and src/ are always included
        paths.push("src/".to_string());
        paths.push(".".to_string());
        Ok(Self { scopes, paths })
    }
}

pub(crate) fn fill_body<T>(t: T, ts: TS) -> TS
where
    // generic to avoid importing both
    // proc_macro and proc_macro2
    TS: From<T>,
{
    // actual inner recursive function
    fn repl(t: TS, ts: &mut Option<TS>) -> TS {
        // iterate all tokens in the token tree
        t.into_iter()
            .map(|tt| {
                // recurse into groups
                if let TT::Group(g) = tt {
                    let mut m = g.stream();
                    let d = g.delimiter();
                    TT::Group(Group::new(
                        d,
                        // we're interested in empty curly
                        // braces or square brackets.
                        if m.is_empty() && (d == Brace || d == Bracket) {
                            // take() ensures we only replace
                            // the first empty brace we see
                            m.extend(ts.take());
                            m
                        } else {
                            // otherwise recurse
                            repl(m, ts)
                        },
                    ))
                } else {
                    // not a group; do not modify
                    tt
                }
            })
            .collect()
    }
    // wrap and invoke
    repl(TS::from(t), &mut Some(ts))
}

pub(crate) fn collect_all(args: Args) -> TS {
    // inner recursive function for directory traversal
    fn recurse(path: &PathBuf, scope: &str, s: &mut HashSet<String>, ts: &mut TS) {
        // don't process the same file twice
        if let Ok(path) = path.canonicalize() {
            if let Some(ps) = path.to_str() {
                if s.contains(ps) {
                    return;
                }
                s.insert(ps.to_string());
            }
        }
        // recurse into directories
        if path.is_dir() {
            if let Ok(entries) = read_dir(path) {
                for e in entries.flatten() {
                    recurse(&e.path(), scope, s, ts);
                }
            }
        } else if is_rust_file(path) {
            // isolate errors to each file
            _ = parse_rs(path, scope, ts);
        }
    }
    // invoke the recursive func
    let mut ts = TS::new();
    for scope in &args.scopes {
        let mut seen = HashSet::new();
        for path in &args.paths {
            recurse(&PathBuf::from(path), scope, &mut seen, &mut ts);
        }
    }
    ts
}

fn is_rust_file(path: &Path) -> bool {
    // extension must exist and be '.rs'
    match path.extension() {
        Some(ext) => ext == "rs",
        None => false,
    }
}

fn parse_rs(path: &PathBuf, scope: &str, ts: &mut TS) -> Result<(), Box<dyn Error>> {
    // open file at path; or return
    let mut f = File::open(path)?;
    // read file contents to string
    let mut b = String::new();
    f.read_to_string(&mut b)?;
    // skip files that don't contain the emit macro
    if b.contains(EMIT) {
        // parse the file, or return
        let tokens = syn::parse_file(&b)?;
        // visit all tokens to gather emitted blocks
        EmitVisitor(scope, ts).fold_file(tokens);
    }
    Ok(())
}

// visitor for emit macros
struct EmitVisitor<'a>(&'a str, &'a mut TS);
impl<'a> Fold for EmitVisitor<'a> {
    fn fold_macro(&mut self, m: Macro) -> Macro {
        // get ident for the macro invocation
        let id = match m.path.segments.last() {
            Some(id) => id,
            None => return m,
        };
        // only consider emit macros
        if id.ident != EMIT {
            return m;
        }
        // iterate scopes
        let mut it = m.tokens.clone().into_iter();
        while match (it.next(), it.next()) {
            // expect pairs of tokens; a key and a group
            (Some(k), Some(TT::Group(g))) => {
                // the group delimiter must be braces
                if g.delimiter() != Brace {
                    panic!("wrong delimiter; {{ block }} expected after scope")
                }
                // only extend with expected scopes
                if k.to_string() == self.0 {
                    self.1.extend(g.stream());
                }
                // keep iterating
                true
            }
            // if both are empty; we're done
            (None, None) => false,
            // other configurations are not allowed
            (_, _) => panic!("emit should contain 'scope {{ block }}' pairs"),
        } { /* nothing left to do in the loop body */ }
        m
    }
}
