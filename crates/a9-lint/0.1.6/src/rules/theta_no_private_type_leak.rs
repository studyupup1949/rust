use syn::{
    Item,
    visit::{self, Visit},
};

use super::{RsRule, Rule, common::has_actor_attr};

struct PrivateTypeVisitor {
    violations: Vec<(usize, String)>,
}

impl<'ast> Visit<'ast> for PrivateTypeVisitor {
    fn visit_type_path(&mut self, tp: &'ast syn::TypePath) {
        for seg in &tp.path.segments {
            if seg.ident == "private" {
                let line = seg.ident.span().start().line;
                self.violations.push((
                    line,
                    "actor message handler must not expose `private::` types; define a public DTO"
                        .to_string(),
                ));
                break;
            }
        }
        visit::visit_type_path(self, tp);
    }
}

pub struct NoPrivateTypeLeak;

impl Rule for NoPrivateTypeLeak {
    fn name(&self) -> &'static str {
        "theta-no-private-type-leak"
    }

    fn description(&self) -> &'static str {
        "actor message return types must not expose `private::` module types; define a public DTO"
    }
}

impl RsRule for NoPrivateTypeLeak {
    fn check(&self, file: &syn::File, _source: &str) -> Vec<(usize, String)> {
        let mut all_violations = vec![];
        for item in &file.items {
            let Item::Impl(impl_item) = item else {
                continue;
            };
            if !has_actor_attr(&impl_item.attrs) {
                continue;
            }
            let mut visitor = PrivateTypeVisitor { violations: vec![] };
            visit::visit_item_impl(&mut visitor, impl_item);
            all_violations.extend(visitor.violations);
        }
        all_violations
    }

    fn try_fix(&self, _source: &str, _file: syn::File) -> Result<String, String> {
        Err("theta-no-private-type-leak: define a public DTO to replace the private type; auto-fix not safe".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        NoPrivateTypeLeak.check(&file, src)
    }

    #[test]
    fn clean_return_types_no_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {
    const _: () = async |msg: Foo| -> Result<Bar, MyError> { todo!() };
}
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn private_path_in_return_type_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {
    const _: () = async |msg: Foo| -> Result<private::Bar, MyError> { todo!() };
}
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].1.contains("private::"));
    }

    #[test]
    fn crate_private_path_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {
    const _: () = async |msg: Foo| -> Result<crate::private::db::Model, MyError> { todo!() };
}
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
    }

    #[test]
    fn no_actor_attr_no_violation() {
        let src = r#"
impl Actor for MyService {
    const _: () = async |msg: Foo| -> Result<private::Bar, MyError> { todo!() };
}
"#;
        assert!(check(src).is_empty());
    }
}
