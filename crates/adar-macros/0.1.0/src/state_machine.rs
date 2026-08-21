use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse::*, *};

pub fn state_enum_macro_inner(
    args: StateMachineArgs,
    mut input: DeriveInput,
) -> syn::Result<TokenStream> {
    let Data::Enum(data_enum) = &mut input.data else {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[StateEnum] macro only supports enums",
        ));
    };

    let ident = &input.ident;
    let visibility = &input.vis;

    let StateMachineArgs {
        args:
            ComplexType {
                generics: args_gen,
                typ: args_type,
                wher: args_where,
            },
        context:
            ComplexType {
                generics: ctx_gen,
                typ: ctx_type,
                wher: ctx_where,
            },
    } = args;

    let combined_gen = combine_generics(args_gen, ctx_gen);
    let combined_where = combine_where(args_where, ctx_where);

    let args_type = args_type.map(|v| quote! {#v}).unwrap_or(quote! {()});
    let ctx_type = ctx_type.map(|v| quote! {#v}).unwrap_or(quote! {()});

    let mut derive = quote! {};
    for attr in &input.attrs {
        if attr.path().is_ident("derive") {
            if let Meta::List(list) = &attr.meta {
                let tokens = &list.tokens;
                derive = quote! {#[derive(#tokens)]};
                break;
            }
        }
    }

    let mut end_state = quote! {};
    let mut variants = vec![];
    let mut enum_variants = vec![];
    let mut variant_structs = vec![];
    for variant in &data_enum.variants {
        let variant_ident = &variant.ident;
        if variant_ident == "EndState" {
            enum_variants.push(quote! {
                #variant_ident(adar::prelude::EndState)
            });
            variant_structs.push(quote! {
                impl Into<#ident> for adar::prelude::EndState {
                    fn into(self) -> #ident {
                        #ident::EndState (adar::prelude::EndState)
                    }
                }
            });
            end_state = quote! {
                impl adar::prelude::HasEndState for #ident {
                    fn is_finished(&self) -> bool {
                        matches!(self, #ident::EndState(_))
                    }
                }
            };
            continue;
        }

        variants.push(quote! {
            #variant_ident
        });

        enum_variants.push(quote! {
            #variant_ident(#variant_ident)
        });

        let meta = quote! {
            impl #combined_gen adar::prelude::StateTypes #combined_gen for #variant_ident #combined_where {
                type States = #ident;
                type Args = #args_type;
                type Context = #ctx_type;
            }

            impl Into<#ident> for #variant_ident {
                fn into(self) -> #ident {
                    #ident::#variant_ident (self)
                }
            }
        };

        match &variant.fields {
            Fields::Named(fields) => {
                let fields_named = fields.named.iter();
                variant_structs.push(quote! {
                    #derive
                    #visibility struct #variant_ident{
                        #(#fields_named),*,
                    }
                    #meta
                });
            }
            Fields::Unit => {
                variant_structs.push(quote! {
                    #derive
                    #visibility struct #variant_ident;
                    #meta
                });
            }
            Fields::Unnamed(fields) => {
                let fields_unnamed = fields.unnamed.iter();
                variant_structs.push(quote! {
                    #derive
                    #visibility struct #variant_ident(#(#fields_unnamed),*,);
                    #meta
                });
            }
        }
    }

    // Patch the enum
    for variant in &mut data_enum.variants {
        let variant_name = &variant.ident;
        let variant_ty = Ident::new(&variant_name.to_string(), variant_name.span());
        variant.fields = Fields::Unnamed(syn::FieldsUnnamed {
            paren_token: Default::default(),
            unnamed: std::iter::once(syn::Field {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                ident: None,
                colon_token: None,
                ty: syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: variant_ty.clone().into(),
                }),
                mutability: FieldMutability::None,
            })
            .collect(),
        });
    }
    Ok(quote! {
        #input

        #(
            #variant_structs
        )*


        impl #combined_gen adar::prelude::StateTypes #combined_gen for #ident #combined_where{
            type States = Self;
            type Args = #args_type;
            type Context = #ctx_type;
        }

        impl #combined_gen adar::prelude::State #combined_gen for #ident #combined_where
        {
            fn on_enter(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
                match self {
                    #(Self::#variants(s)=> #variants::on_enter(s, args, context)),*,
                    _=>(),
                }
            }

            fn on_update(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) -> Option<Self::States> {
                match self {
                    #(Self::#variants(s)=> #variants::on_update(s, args, context)),*,
                    _=>None,
                }
            }

            fn on_leave(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
                match self {
                    #(Self::#variants(s)=> #variants::on_leave(s, args, context)),*,
                    _=>(),
                }
            }
        }

        #end_state
    }
    .into())
}

#[derive(Default, Debug)]
pub struct ComplexType {
    pub generics: Option<Generics>,
    pub typ: Option<Type>,
    pub wher: Option<WhereClause>,
}

#[derive(Default, Debug)]
pub struct StateMachineArgs {
    pub args: ComplexType,
    pub context: ComplexType,
}

impl Parse for StateMachineArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut result = StateMachineArgs::default();
        let mut first = true;
        while !input.is_empty() {
            if !first {
                input.parse::<Token![,]>()?;
            }
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if ident == "args" {
                result.args = Self::parse_type(&input)?;
            } else if ident == "context" {
                result.context = Self::parse_type(&input)?;
            } else {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Invalid identifier: {}", ident),
                ));
            }

            first = false;
        }

        Ok(result)
    }
}

impl StateMachineArgs {
    fn parse_type(input: &syn::parse::ParseStream) -> syn::Result<ComplexType> {
        Ok(ComplexType {
            generics: if input.lookahead1().peek(Token![for]) {
                input.parse::<Token![for]>()?;
                Some(input.parse()?)
            } else {
                None
            },
            typ: Some(input.parse()?),
            wher: if input.peek(Token![where]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}

pub fn combine_where(a: Option<WhereClause>, b: Option<WhereClause>) -> Option<WhereClause> {
    match (a, b) {
        (None, None) => None,
        (Some(w), None) | (None, Some(w)) => Some(w),
        (Some(mut w1), Some(w2)) => {
            w1.predicates.extend(w2.predicates);
            Some(w1)
        }
    }
}

pub fn combine_generics(a: Option<Generics>, b: Option<Generics>) -> Option<Generics> {
    match (a, b) {
        (None, None) => None,
        (Some(g), None) | (None, Some(g)) => Some(g),
        (Some(mut g1), Some(g2)) => {
            g1.params.extend(g2.params);
            Some(g1)
        }
    }
}
