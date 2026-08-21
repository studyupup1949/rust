use syn::{File, Item};

use super::common::has_actor_attr;
use crate::{Rule, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl Rule for UnitRule {
    fn name(&self) -> &'static str {
        "theta-actors-at-bottom"
    }

    fn description(&self) -> &'static str {
        "`#[actor]` impl blocks must all appear at the bottom; no non-actor item may follow one"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        let first_actor_pos = ast.items.iter().position(is_actor_impl);

        let Some(first_actor_pos) = first_actor_pos else {
            return vec![];
        };

        ast.items[first_actor_pos + 1..]
            .iter()
            .filter_map(|item| {
                if is_actor_impl(item) {
                    return None;
                }

                Some(Violation {
                    line: item_start_line(item),
                    message: "non-actor item must not appear after an `#[actor]` impl block"
                        .to_string(),
                    fixable: true,
                })
            })
            .collect()
    }

    fn fix(&self, mut ast: File) -> File {
        let mut non_actors = Vec::new();
        let mut actors = Vec::new();

        for item in ast.items.drain(..) {
            if is_actor_impl(&item) {
                actors.push(item);
            } else {
                non_actors.push(item);
            }
        }

        non_actors.extend(actors);
        ast.items = non_actors;

        ast
    }
}

fn is_actor_impl(item: &Item) -> bool {
    matches!(item, Item::Impl(i) if has_actor_attr(&i.attrs))
}

fn item_start_line(item: &Item) -> usize {
    match item {
        Item::Struct(s) => s.struct_token.span.start().line,
        Item::Enum(e) => e.enum_token.span.start().line,
        Item::Fn(f) => f.sig.fn_token.span.start().line,
        Item::Impl(i) => i.impl_token.span.start().line,
        Item::Mod(m) => m.mod_token.span.start().line,
        Item::Use(u) => u.use_token.span.start().line,
        Item::Const(c) => c.const_token.span.start().line,
        Item::Type(t) => t.type_token.span.start().line,
        Item::Trait(t) => t.trait_token.span.start().line,
        Item::Static(s) => s.static_token.span.start().line,
        _ => 0,
    }
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
    fn no_actor_ok() {
        let src = "pub struct Foo;\npub struct Bar;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn single_actor_at_bottom_ok() {
        let src = "pub struct Foo;\n#[actor(\"u\")]\nimpl Actor for Foo {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn single_actor_not_last_is_violation() {
        let src = "#[actor(\"u\")]\nimpl Actor for Foo {}\npub struct Bar;\n";

        assert_eq!(detect(src).len(), 1);
    }

    #[test]
    fn multiple_actors_at_bottom_ok() {
        let src = "pub struct Foo;\npub struct Bar;\n#[actor(\"u\")]\nimpl Actor for Foo {}\n#[actor(\"v\")]\nimpl Actor for Bar {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn non_actor_between_actors_is_violation() {
        let src = "#[actor(\"u\")]\nimpl Actor for Foo {}\npub struct Mid;\n#[actor(\"v\")]\nimpl Actor for Bar {}\n";

        assert_eq!(detect(src).len(), 1);
    }

    #[test]
    fn fix_moves_actors_to_bottom() {
        let src = "#[actor(\"u\")]\nimpl Actor for Foo {}\npub struct Bar;\n";
        let fixed = fix(src);

        assert!(
            fixed.find("struct Bar").unwrap() < fixed.find("impl Actor").unwrap(),
            "struct should come before actor: {fixed}"
        );
    }

    #[test]
    fn fix_then_detect_clean() {
        let src = "#[actor(\"u\")]\nimpl Actor for Foo {}\npub struct Bar;\n";
        let fixed = fix(src);
        let v: Vec<_> = detect(&fixed).into_iter().filter(|v| v.fixable).collect();

        assert!(v.is_empty(), "fixable violations remain: {v:?}");
    }
}
