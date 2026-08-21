use syn::{Fields, Item};

use super::{
    RsRule, Rule,
    common::{actor_self_type_name, has_actor_attr, is_cfg_feature},
};

pub struct ActorFieldsGated;

impl Rule for ActorFieldsGated {
    fn name(&self) -> &'static str {
        "theta-actor-fields-gated"
    }

    fn description(&self) -> &'static str {
        "all fields of the actor struct must be gated with `#[cfg(feature = \"private\")]`"
    }
}

impl RsRule for ActorFieldsGated {
    fn check(&self, file: &syn::File, _source: &str) -> Vec<(usize, String)> {
        let actor_impl = file.items.iter().find_map(|item| {
            let Item::Impl(i) = item else { return None };
            if has_actor_attr(&i.attrs) {
                Some(i)
            } else {
                None
            }
        });
        let Some(actor_impl) = actor_impl else {
            return vec![];
        };
        let Some(struct_name) = actor_self_type_name(actor_impl) else {
            return vec![];
        };

        let actor_struct = file.items.iter().find_map(|item| {
            let Item::Struct(s) = item else { return None };
            if s.ident == struct_name {
                Some(s)
            } else {
                None
            }
        });
        let Some(actor_struct) = actor_struct else {
            return vec![];
        };

        let Fields::Named(named) = &actor_struct.fields else {
            return vec![];
        };

        let mut violations = vec![];
        for field in &named.named {
            if !field.attrs.iter().any(|a| is_cfg_feature(a, "private")) {
                let name = field
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                let line = field
                    .ident
                    .as_ref()
                    .map(|i| i.span().start().line)
                    .unwrap_or(1);
                violations.push((
                    line,
                    format!(
                        "actor struct field `{name}` must be gated with `#[cfg(feature = \"private\")]`"
                    ),
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
        ActorFieldsGated.check(&file, src)
    }

    #[test]
    fn all_gated_no_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {}

pub struct MyService {
    #[cfg(feature = "private")]
    pub(crate) db: String,
}
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn ungated_field_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {}

pub struct MyService {
    pub db: String,
}
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].1.contains("db"));
    }

    #[test]
    fn wrong_feature_name_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {}

pub struct MyService {
    #[cfg(feature = "implementation")]
    pub(crate) db: String,
}
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
    }

    #[test]
    fn empty_struct_no_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {}

pub struct MyService {}
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn no_actor_impl_no_violation() {
        let src = r#"
pub struct MyService {
    pub db: String,
}
"#;
        assert!(check(src).is_empty());
    }
}
