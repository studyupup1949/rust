use syn::Item;

use super::{
    RsRule, Rule,
    common::{has_actor_attr, rejoin_sources, split_item_sources},
};

pub struct ActorsAtBottom;

impl Rule for ActorsAtBottom {
    fn name(&self) -> &'static str {
        "theta-actors-at-bottom"
    }

    fn description(&self) -> &'static str {
        "`#[actor]` impl blocks must all appear at the bottom; no non-actor item may follow one"
    }
}

impl RsRule for ActorsAtBottom {
    fn check(&self, file: &syn::File, _source: &str) -> Vec<(usize, String)> {
        // Find the position of the first #[actor] impl.
        let first_actor_pos = file.items.iter().position(|item| {
            let Item::Impl(i) = item else { return false };
            has_actor_attr(&i.attrs)
        });
        let Some(first_actor_pos) = first_actor_pos else {
            return vec![];
        };

        // Every item after the first actor impl must also be an actor impl.
        file.items[first_actor_pos + 1..]
            .iter()
            .filter_map(|item| {
                let is_actor = matches!(item, Item::Impl(i) if has_actor_attr(&i.attrs));
                if is_actor {
                    return None;
                }
                let line = match item {
                    Item::Struct(s) => s.struct_token.span.start().line,
                    Item::Enum(e) => e.enum_token.span.start().line,
                    Item::Fn(f) => f.sig.fn_token.span.start().line,
                    Item::Impl(i) => i.impl_token.span.start().line,
                    Item::Mod(m) => m.mod_token.span.start().line,
                    Item::Use(u) => u.use_token.span.start().line,
                    _ => 0,
                };
                Some((
                    line,
                    "non-actor item must not appear after an `#[actor]` impl block".to_string(),
                ))
            })
            .collect()
    }

    fn has_fixer(&self) -> bool {
        true
    }

    fn try_fix(&self, source: &str, file: syn::File) -> Result<String, String> {
        if self.check(&file, source).is_empty() {
            return Err("no violations to fix".into());
        }
        let (preamble, texts) = split_item_sources(&file.items, source);
        let mut actors: Vec<String> = Vec::new();
        let mut non_actors: Vec<String> = Vec::new();
        for (item, text) in file.items.iter().zip(texts) {
            if matches!(item, Item::Impl(i) if has_actor_attr(&i.attrs)) {
                actors.push(text);
            } else {
                non_actors.push(text);
            }
        }
        non_actors.extend(actors);
        Ok(rejoin_sources(
            &preamble,
            &non_actors,
            source.ends_with('\n'),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        ActorsAtBottom.check(&file, src)
    }

    #[test]
    fn no_actor_ok() {
        let src = "pub struct Foo; pub struct Bar;";
        assert!(check(src).is_empty());
    }

    #[test]
    fn single_actor_at_bottom_ok() {
        let src = "pub struct Foo; #[actor(\"u\")] impl Actor for Foo {}";
        assert!(check(src).is_empty());
    }

    #[test]
    fn single_actor_not_last_is_violation() {
        let src = "#[actor(\"u\")] impl Actor for Foo {} pub struct Bar;";
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn multiple_actors_at_bottom_ok() {
        let src = "pub struct Foo; pub struct Bar; #[actor(\"u\")] impl Actor for Foo {} #[actor(\"v\")] impl Actor for Bar {}";
        assert!(check(src).is_empty());
    }

    #[test]
    fn non_actor_between_actors_is_violation() {
        let src = "#[actor(\"u\")] impl Actor for Foo {} pub struct Mid; #[actor(\"v\")] impl Actor for Bar {}";
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn multiple_non_actors_after_actor_each_flagged() {
        let src = "#[actor(\"u\")] impl Actor for Foo {} pub struct A; pub struct B;";
        assert_eq!(check(src).len(), 2);
    }
}
