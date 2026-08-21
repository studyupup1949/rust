use syn::{Fields, Item};

use super::common::{actor_self_type_name, has_actor_attr, is_cfg_feature};
use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "theta-actor-fields-gated"
    }

    fn description(&self) -> &'static str {
        "all fields of the actor struct must be gated with `#[cfg(feature = \"private\")]`"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &syn::File) -> Vec<Violation> {
        let Some(struct_name) = find_actor_struct_name(ast) else {
            return vec![];
        };
        let actor_struct = ast.items.iter().find_map(|item| {
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

        named
            .named
            .iter()
            .filter(|f| field_needs_gate(f))
            .map(|f| {
                let name = f
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let line = f.ident.as_ref().map_or(1, |i| i.span().start().line);

                Violation {
                    line,
                    message: format!(
                        "actor struct field `{name}` must be gated with `#[cfg(feature = \"private\")]`"
                    ),
                    fixable: true,
                }
            })
            .collect()
    }

    fn fix(&self, mut ast: syn::File) -> syn::File {
        let Some(struct_name) = find_actor_struct_name(&ast) else {
            return ast;
        };

        for item in &mut ast.items {
            let Item::Struct(s) = item else { continue };

            if s.ident != struct_name {
                continue;
            }

            let Fields::Named(named) = &mut s.fields else {
                continue;
            };

            for field in &mut named.named {
                if field_needs_gate(field) {
                    field.attrs.push(make_cfg_private_attr());
                }
            }
        }

        ast
    }
}

fn find_actor_struct_name(file: &syn::File) -> Option<String> {
    file.items.iter().find_map(|item| {
        let Item::Impl(i) = item else { return None };

        if has_actor_attr(&i.attrs) {
            actor_self_type_name(i)
        } else {
            None
        }
    })
}

fn field_needs_gate(field: &syn::Field) -> bool {
    !field.attrs.iter().any(|a| is_cfg_feature(a, "private"))
}

fn make_cfg_private_attr() -> syn::Attribute {
    syn::parse_quote!(#[cfg(feature = "private")])
}

#[cfg(test)]
mod tests {
    use a9_prettyplease::unparse;

    use super::*;

    fn detect(src: &str) -> Vec<Violation> {
        let file = syn::parse_file(src).unwrap();

        UnitRule.detect(&file)
    }

    fn fix(src: &str) -> String {
        let file = syn::parse_file(src).unwrap();
        let fixed = UnitRule.fix(file);

        unparse(&fixed)
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

        assert!(detect(src).is_empty());
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
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("db"));
        assert!(vs[0].fixable);
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
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
    }

    #[test]
    fn empty_struct_no_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {}

pub struct MyService {}
"#;

        assert!(detect(src).is_empty());
    }

    #[test]
    fn no_actor_impl_no_violation() {
        let src = "pub struct MyService {\n    pub db: String,\n}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn fix_adds_cfg_attr_to_fields() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {}

pub struct MyService {
    pub db: String,
}
"#;
        let fixed = fix(src);

        assert!(
            fixed.contains("cfg(feature = \"private\")"),
            "should add cfg attr: {fixed}"
        );
    }

    #[test]
    fn fix_then_detect_clean() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {}

pub struct MyService {
    pub db: String,
}
"#;
        let fixed = fix(src);
        let v: Vec<_> = detect(&fixed).into_iter().filter(|v| v.fixable).collect();

        assert!(v.is_empty(), "fixable violations remain: {v:?}");
    }
}
