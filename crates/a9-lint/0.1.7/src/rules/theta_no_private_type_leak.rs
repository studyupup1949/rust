use syn::{
    Item,
    visit::{self, Visit},
};

use super::common::has_actor_attr;
use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl<'ast> Visit<'ast> for PrivateTypeVisitor {
    fn visit_type_path(&mut self, tp: &'ast syn::TypePath) {
        for seg in &tp.path.segments {
            if seg.ident == "private" {
                let line = seg.ident.span().start().line;

                self.violations
                    .push(Violation {
                        line,
                        message: "actor message handler must not expose `private::` types; define a public DTO"
                            .to_string(),
                        fixable: false,
                    });

                break;
            }
        }

        visit::visit_type_path(self, tp);
    }
}

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "theta-no-private-type-leak"
    }

    fn description(&self) -> &'static str {
        "actor message return types must not expose `private::` module types; define a public DTO"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &syn::File) -> Vec<Violation> {
        let mut all = vec![];

        for item in &ast.items {
            let Item::Impl(impl_item) = item else {
                continue;
            };

            if !has_actor_attr(&impl_item.attrs) {
                continue;
            }

            let mut visitor = PrivateTypeVisitor { violations: vec![] };

            visit::visit_item_impl(&mut visitor, impl_item);
            all.extend(visitor.violations);
        }

        all
    }
}

struct PrivateTypeVisitor {
    violations: Vec<Violation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(src: &str) -> Vec<Violation> {
        let file = syn::parse_file(src).unwrap();

        UnitRule.detect(&file)
    }

    #[test]
    fn clean_return_types_no_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {
    const _: () = async |msg: Foo| -> Result<Bar, MyError> { todo!() };
}
"#;

        assert!(detect(src).is_empty());
    }

    #[test]
    fn private_path_in_return_type_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {
    const _: () = async |msg: Foo| -> Result<private::Bar, MyError> { todo!() };
}
"#;
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("private::"));
    }

    #[test]
    fn crate_private_path_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for MyService {
    const _: () = async |msg: Foo| -> Result<crate::private::db::Model, MyError> { todo!() };
}
"#;
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
    }

    #[test]
    fn no_actor_attr_no_violation() {
        let src = r#"
impl Actor for MyService {
    const _: () = async |msg: Foo| -> Result<private::Bar, MyError> { todo!() };
}
"#;

        assert!(detect(src).is_empty());
    }
}
