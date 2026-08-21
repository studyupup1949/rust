use syn::Item;

use super::{RsRule, Rule, common::is_cfg_feature};

pub struct ActorPrivateGate;

impl Rule for ActorPrivateGate {
    fn name(&self) -> &'static str {
        "theta-actor-private-gate"
    }

    fn description(&self) -> &'static str {
        "declaration-only `mod private;` must be gated with `#[cfg(feature = \"private\")]`"
    }
}

impl RsRule for ActorPrivateGate {
    fn check(&self, file: &syn::File, _source: &str) -> Vec<(usize, String)> {
        let mut violations = vec![];
        for item in &file.items {
            let Item::Mod(m) = item else { continue };
            if m.content.is_some() || m.ident != "private" {
                continue;
            }
            if !m.attrs.iter().any(|a| is_cfg_feature(a, "private")) {
                let line = m.mod_token.span.start().line;
                violations.push((
                    line,
                    "`mod private;` must be gated with `#[cfg(feature = \"private\")]`".to_string(),
                ));
            }
        }
        violations
    }

    fn has_fixer(&self) -> bool {
        true
    }

    fn try_fix(&self, source: &str, file: syn::File) -> Result<String, String> {
        let violations = self.check(&file, source);
        if violations.is_empty() {
            return Err("no violations to fix".into());
        }
        let bad: std::collections::HashSet<usize> = violations.iter().map(|(l, _)| *l).collect();
        let lines: Vec<&str> = source.lines().collect();
        let mut out: Vec<String> = Vec::with_capacity(lines.len() + violations.len());
        for (i, &line) in lines.iter().enumerate() {
            if bad.contains(&(i + 1)) {
                let indent: String = line
                    .chars()
                    .take_while(|c| c.is_ascii_whitespace())
                    .collect();
                out.push(format!("{indent}#[cfg(feature = \"private\")]"));
            }
            out.push(line.to_string());
        }
        let mut new_src = out.join("\n");
        if source.ends_with('\n') {
            new_src.push('\n');
        }
        Ok(new_src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        ActorPrivateGate.check(&file, src)
    }

    #[test]
    fn correct_private_feature_no_violation() {
        let src = r#"
#[cfg(feature = "private")]
pub(crate) mod private;
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn missing_cfg_violation() {
        let src = r#"
pub(crate) mod private;
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].1.contains("gated with"));
    }

    #[test]
    fn wrong_feature_name_violation() {
        let src = r#"
#[cfg(feature = "implementation")]
pub(crate) mod private;
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
    }

    #[test]
    fn no_mod_private_no_violation() {
        assert!(check("pub struct Foo;").is_empty());
    }

    #[test]
    fn inline_private_mod_no_violation() {
        // Inline `mod private { }` is not a declaration — rule does not apply.
        let src = r#"
mod private {
    pub struct Inner;
}
"#;
        assert!(check(src).is_empty());
    }
}
