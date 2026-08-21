//!
//! These only have an effect when compiling with debug feature
//! `WINNOW_TRACE=all`      (default)
//! `WINNOW_TRACE=poe_data_tools` (enable for a crate)
//! `WINNOW_TRACE=winnow`
//! `WINNOW_TRACE=`            (disable all logging)
//! `WINNOW_TRACE=poe_data_tools::file_parsers::ao` (enable for a module)
//! `WINNOW_TRACE=poe_data_tools::file_parsers::ao=children` (enable for a parser and all child
//! parsers called by it)
//! `WINNOW_TRACE=winnow,poe_data_tools::file_parsers` (multiple filters)
//!

use lazy_static::lazy_static;
use std::fmt::Display;

lazy_static! {
    pub(crate) static ref FILTERS: Filters = Filters(Filter::from_env());
}

#[derive(Debug, PartialEq, Eq)]
enum Filter {
    All,
    Id(String),
    Children(String),
}

impl Filter {
    /// Whether the provided indentifier should be allowed
    /// Identifier should be a fully qualified path to the parser.
    /// Eg `winnow::combinators::preceded`
    fn enabled(&self, identifier: &str) -> bool {
        use Filter::{All, Children, Id};
        match self {
            All => true,
            // TODO: Structured path?
            Id(id) | Children(id) => identifier.starts_with(id),
        }
    }

    /// Whether we should show child parsers for the given identifier
    fn show_children(&self, identifier: &str) -> bool {
        use Filter::{All, Children, Id};
        match self {
            All | Id(_) => false,
            // TODO: Structured path?
            Children(id) => identifier.starts_with(id),
        }
    }

    fn parse_env(env: &str) -> Vec<Self> {
        // TODO: Validation
        env.split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| match part.split_once("=") {
                None => {
                    if part == "all" {
                        Filter::All
                    } else {
                        Filter::Id(part.to_owned())
                    }
                }
                Some((id, "children")) => Filter::Children(id.to_owned()),
                Some((_, flag)) => panic!("Invalid filter flag: {flag:?}"),
            })
            .collect()
    }

    fn from_env() -> Vec<Self> {
        let Ok(env_str) = std::env::var("WINNOW_TRACE") else {
            // ENV not set, so let everything through
            return vec![Self::All];
        };

        Self::parse_env(&env_str)
    }
}

pub(crate) struct Filters(Vec<Filter>);

impl Filters {
    pub(crate) fn enabled(&self, identifier: impl Display) -> bool {
        let identifier = format!("{identifier}");
        self.0.iter().any(|f| f.enabled(&identifier))
    }

    pub(crate) fn show_children(&self, identifier: impl Display) -> bool {
        let identifier = format!("{identifier}");
        self.0.iter().any(|f| f.show_children(&identifier))
    }
}

/// When a filter has show children enabled, this tracks the current stack of children in case
/// there are nested ones
static CHILDREN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub(crate) struct ShowChildren(());

impl ShowChildren {
    /// Increment the children stack and grab a drop guard
    pub(crate) fn new() -> Self {
        // TODO: Might not need atomic if we're not actually using the value?
        CHILDREN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self(())
    }

    /// Check if we're currently showing children
    pub(crate) fn enabled() -> bool {
        CHILDREN.load(std::sync::atomic::Ordering::SeqCst) > 0
    }
}

impl Drop for ShowChildren {
    fn drop(&mut self) {
        CHILDREN.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_all() {
        let filter = Filter::All;

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(filter.enabled("winnow::mutli::repeat"));
        assert!(filter.enabled("winnow::binary::be_u16"));
        assert!(filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_filter_crate() {
        let filter = Filter::Id("winnow".to_owned());

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(filter.enabled("winnow::mutli::repeat"));
        assert!(filter.enabled("winnow::binary::be_u16"));
        assert!(!filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_filter_module() {
        let filter = Filter::Id("winnow::combinators".to_owned());

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(!filter.enabled("winnow::mutli::repeat"));
        assert!(!filter.enabled("winnow::binary::be_u16"));
        assert!(!filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_env_filters_all() {
        let env_str = "all";

        let filters = Filter::parse_env(env_str);

        assert_eq!(filters, [Filter::All]);
    }

    #[test]
    fn test_env_filters_multi() {
        let env_str = "winnow::combinator,winnow::combinator::multi::repeat";

        let filters = Filter::parse_env(env_str);

        assert_eq!(
            filters,
            [
                Filter::Id("winnow::combinator".to_owned()),
                Filter::Id("winnow::combinator::multi::repeat".to_owned())
            ]
        );
    }

    #[test]
    fn test_env_filters_none() {
        let env_str = "";

        let filters = Filter::parse_env(env_str);

        assert_eq!(filters, []);
    }

    #[test]
    fn test_env_filters_children() {
        let env_str = "winnow::combinators::preceded=children";

        let filters = Filter::parse_env(env_str);

        assert_eq!(
            filters,
            [Filter::Children("winnow::combinators::preceded".to_owned())]
        );
    }

    #[test]
    fn test_filters_all() {
        let filter = Filters(vec![Filter::All]);

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(filter.enabled("winnow::mutli::repeat"));
        assert!(filter.enabled("winnow::binary::be_u16"));
        assert!(filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_filters_multi() {
        let filter = Filters(vec![
            Filter::Id("winnow::combinators".to_owned()),
            Filter::Id("winnow::binary::be_u16".to_owned()),
        ]);

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(!filter.enabled("winnow::mutli::repeat"));
        assert!(filter.enabled("winnow::binary::be_u16"));
        assert!(!filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_filters_none() {
        let filter = Filters(vec![]);

        assert!(!filter.enabled("winnow::combinators::preceded"));
        assert!(!filter.enabled("winnow::combinators::terminated"));
        assert!(!filter.enabled("winnow::mutli::repeat"));
        assert!(!filter.enabled("winnow::binary::be_u16"));
        assert!(!filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_children() {
        assert_eq!(CHILDREN.load(std::sync::atomic::Ordering::SeqCst), 0);
        {
            let _guard = ShowChildren::new();
            assert_eq!(CHILDREN.load(std::sync::atomic::Ordering::SeqCst), 1);
            {
                let _guard = ShowChildren::new();
                assert_eq!(CHILDREN.load(std::sync::atomic::Ordering::SeqCst), 2);
            }
            assert_eq!(CHILDREN.load(std::sync::atomic::Ordering::SeqCst), 1);
        }
        assert_eq!(CHILDREN.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
}
