use syn::{Attribute, Result};

#[allow(dead_code)]
pub struct AttrIdentFilter {
    pub base_attrs: Vec<Attribute>,
    matches: Vec<Attribute>,
    misses: Vec<Attribute>
}
#[allow(dead_code)]
impl AttrIdentFilter {
    pub fn refilter(&self, references: Vec<&'static str>) -> Self {
        Self::filter(self.base_attrs.clone(), references)
    }

    fn filter(attrs: Vec<Attribute>, references: Vec<&'static str>) -> Self {
        let match_fn = |attr: &&Attribute| {
            let mut output = false;
            for ident in &references {
                match &attr.meta {
                    syn::Meta::Path(path) if path.is_ident(ident) => output = true,
                    _ => continue
                }
            }
            output
        };

        let matches = attrs
            .iter()
            .filter(
                |attr| match_fn(attr)
            )
            .map(
                |attr| attr.clone()
            )
            .collect::<Vec<Attribute>>();

        let misses = attrs
            .iter()
            .filter(
                |attr| !match_fn(attr)
            )
            .map(
                |attr| attr.clone()
            )
            .collect::<Vec<Attribute>>();

        Self {
            base_attrs: attrs,
            matches,
            misses
        }
    }

    pub fn parse_filter(input: &syn::parse::ParseStream, references: Vec<&'static str>) -> Result<Self> {
        let base_attrs: Vec<Attribute> = Attribute::parse_outer(input)?;
        Ok(Self::filter(base_attrs, references))
    }

    pub fn matches(&self) -> Vec<Attribute> {
        self.matches.clone()
    }

    pub fn misses(&self) -> Vec<Attribute> {
        self.misses.clone()
    }

    pub fn match_against(&self, references: &'static str, allow_duplicate: bool) -> bool {
        if allow_duplicate {
            self.refilter(vec![references]).matches().len() >= 1
        } else {
            self.refilter(vec![references]).matches().len() == 1
        }
    }
}