use syn::Item;

use super::{RsRule, Rule, common::has_actor_attr};

pub struct ActorAtBottom;

impl Rule for ActorAtBottom {
    fn name(&self) -> &'static str {
        "theta-actor-at-bottom"
    }

    fn description(&self) -> &'static str {
        "the `#[actor]` impl block must be the last item in the file"
    }
}

impl RsRule for ActorAtBottom {
    fn check(&self, file: &syn::File) -> Vec<(usize, String)> {
        let actor_pos = file.items.iter().position(|item| {
            let Item::Impl(i) = item else { return false };
            has_actor_attr(&i.attrs)
        });
        let Some(actor_pos) = actor_pos else {
            return vec![];
        };

        if actor_pos == file.items.len() - 1 {
            return vec![];
        }

        let Item::Impl(impl_item) = &file.items[actor_pos] else {
            unreachable!()
        };
        let line = impl_item.impl_token.span.start().line;
        let trailing = file.items.len() - 1 - actor_pos;
        vec![(
            line,
            format!(
                "`#[actor]` impl must be the last item in the file ({trailing} item(s) follow it)"
            ),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        ActorAtBottom.check(&file)
    }

    #[test]
    fn actor_impl_last_no_violation() {
        let src = r#"
pub struct Foo;

pub enum Bar {}

#[actor("uuid")]
impl Actor for Foo {}
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn items_after_actor_impl_violation() {
        let src = r#"
#[actor("uuid")]
impl Actor for Foo {}

pub struct Bar;
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].1.contains("last item"));
    }

    #[test]
    fn no_actor_impl_no_violation() {
        let src = r#"
pub struct Foo;
pub struct Bar;
"#;
        assert!(check(src).is_empty());
    }
}
