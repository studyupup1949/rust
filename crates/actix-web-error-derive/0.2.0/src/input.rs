use crate::{attr::Attrs, generics::TypeParams};
use proc_macro2::Span;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Generics, Ident, Index, Member, Result,
    Type,
};

pub enum Input<'a> {
    Struct(Struct<'a>),
    Enum(Enum<'a>),
}

pub struct Struct<'a> {
    pub original: &'a DeriveInput,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
}

pub struct Enum<'a> {
    pub original: &'a DeriveInput,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub generics: &'a Generics,
    pub variants: Vec<Variant<'a>>,
}

pub struct Variant<'a> {
    pub original: &'a syn::Variant,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub fields: Vec<Field<'a>>,
}

pub struct Field<'a> {
    pub original: &'a syn::Field,
    pub attrs: Attrs<'a>,
    pub member: Member,
    pub ty: &'a Type,
    pub contains_generic: bool,
}

impl<'a> Input<'a> {
    pub fn from_syn(node: &'a DeriveInput) -> Result<Self> {
        match &node.data {
            Data::Struct(data) => Struct::from_syn(node, data).map(Input::Struct),
            Data::Enum(data) => Enum::from_syn(node, data).map(Input::Enum),
            Data::Union(_) => Err(Error::new_spanned(
                node,
                "union as errors are not supported",
            )),
        }
    }
}

impl<'a> Struct<'a> {
    fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        let attrs = Attrs::get(&node.attrs)?;
        let scope = TypeParams::new(&node.generics);
        let span = attrs.span().unwrap_or_else(Span::call_site);
        let fields = Field::multiple_from_syn(&data.fields, &scope, span)?;
        Ok(Struct {
            original: node,
            attrs,
            ident: node.ident.clone(),
            generics: &node.generics,
            fields,
        })
    }
}

impl<'a> Enum<'a> {
    fn from_syn(node: &'a DeriveInput, data: &'a DataEnum) -> Result<Self> {
        let attrs = Attrs::get(&node.attrs)?;
        let scope = TypeParams::new(&node.generics);
        let span = attrs.span().unwrap_or_else(Span::call_site);
        let variants = data
            .variants
            .iter()
            .map(|node| {
                let mut variant = Variant::from_syn(node, &scope, span)?;
                if let display @ None = &mut variant.attrs.status {
                    *display = attrs.status.clone();
                }
                Ok(variant)
            })
            .collect::<Result<_>>()?;
        Ok(Enum {
            original: node,
            attrs,
            ident: node.ident.clone(),
            generics: &node.generics,
            variants,
        })
    }
}

impl<'a> Variant<'a> {
    fn from_syn(node: &'a syn::Variant, scope: &TypeParams<'a>, span: Span) -> Result<Self> {
        let attrs = Attrs::get(&node.attrs)?;
        let span = attrs.span().unwrap_or(span);
        Ok(Variant {
            original: node,
            attrs,
            ident: node.ident.clone(),
            fields: Field::multiple_from_syn(&node.fields, scope, span)?,
        })
    }
}

impl<'a> Field<'a> {
    fn multiple_from_syn(
        fields: &'a Fields,
        scope: &TypeParams<'a>,
        span: Span,
    ) -> Result<Vec<Self>> {
        fields
            .iter()
            .enumerate()
            .map(|(i, field)| Field::from_syn(i, field, scope, span))
            .collect()
    }

    fn from_syn(
        i: usize,
        node: &'a syn::Field,
        scope: &TypeParams<'a>,
        span: Span,
    ) -> Result<Self> {
        Ok(Field {
            original: node,
            attrs: Attrs::get(&node.attrs)?,
            member: node.ident.clone().map_or_else(
                || {
                    Member::Unnamed(Index {
                        index: i.try_into().unwrap(),
                        span,
                    })
                },
                Member::Named,
            ),
            ty: &node.ty,
            contains_generic: scope.intersects(&node.ty),
        })
    }
}
