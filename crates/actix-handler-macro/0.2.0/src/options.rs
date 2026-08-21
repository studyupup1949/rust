use quote::ToTokens;
use syn::{AttributeArgs, Lit, Meta, NestedMeta};

pub struct Options {
    pub(crate) trait_name: Option<String>,
    pub(crate) no_trait_decl: bool,
    pub(crate) no_trait_impl: bool,
    pub(crate) use_recipient: bool,
}

pub fn parse_options(args: AttributeArgs) -> Options {
    let mut options = Options {
        trait_name: None,
        use_recipient: false,
        no_trait_decl: false,
        no_trait_impl: false,
    };

    for arg in args {
        match arg {
            NestedMeta::Meta(meta) => match meta {
                Meta::Path(path) => match path.to_token_stream().to_string().as_str() {
                    "use_recipient" => {
                        options.use_recipient = true;
                    }
                    "no_trait_decl" => {
                        options.no_trait_decl = true;
                    }
                    "no_trait_impl" => {
                        options.no_trait_impl = true;
                    }
                    _ => {}
                },
                Meta::NameValue(name_value) => {
                    if let "trait_name" = name_value.path.to_token_stream().to_string().as_str() {
                        if let Lit::Str(trait_name) = name_value.lit {
                            options.trait_name = Some(trait_name.value());
                        }
                    }
                }
                Meta::List(_) => {}
            },
            NestedMeta::Lit(_) => {}
        }
    }

    options
}
