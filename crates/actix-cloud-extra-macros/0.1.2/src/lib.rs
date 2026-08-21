#![cfg_attr(docsrs, feature(doc_cfg))]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ImplItem, ItemImpl, parse_macro_input, parse_quote};

/// Implement default viewer.
///
/// # Examples
/// ```ignore
/// pub struct UserViewer;
///
/// #[default_viewer]
/// impl UserViewer {}
/// ```
#[proc_macro_attribute]
pub fn default_viewer(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Ident);
    let mut input = parse_macro_input!(input as ItemImpl);
    let func: Vec<String> = input
        .items
        .iter()
        .map(|x| {
            if let ImplItem::Fn(x) = x {
                x.sig.ident.to_string()
            } else {
                String::new()
            }
        })
        .collect();
    let contains = |name: &str| func.iter().any(|x| x == name);
    if !contains("find") {
        input.items.push(parse_quote! {
            pub async fn find<C>(
                db: &C,
                cond: actix_cloud_extra::api::Condition,
            ) -> anyhow::Result<(Vec<#attr::Model>, u64)>
            where
                C: sea_orm::ConnectionTrait
            {
                cond.select_page(#attr::Entity::find(), db).await
            }
        });
    }
    if !contains("find_by_id") {
        input.items.push(parse_quote! {
            pub async fn find_by_id<C>(db: &C, id: &actix_cloud_extra::HyUuid) -> anyhow::Result<Option<#attr::Model>>
            where
                C: sea_orm::ConnectionTrait,
            {
                #attr::Entity::find_by_id(id.to_owned())
                    .one(db)
                    .await
                    .map_err(Into::into)
            }
        });
    }
    if !contains("delete_all") {
        input.items.push(parse_quote! {
            pub async fn delete_all<C>(db: &C) -> anyhow::Result<u64>
            where
                C: sea_orm::ConnectionTrait,
            {
                #attr::Entity::delete_many()
                    .exec(db)
                    .await
                    .map(|x| x.rows_affected)
                    .map_err(Into::into)
            }
        });
    }
    if !contains("delete") {
        input.items.push(parse_quote! {
            pub async fn delete<C>(db: &C, id: &[actix_cloud_extra::HyUuid]) -> anyhow::Result<u64>
            where
                C: sea_orm::ConnectionTrait,
            {
                #attr::Entity::delete_many()
                    .filter(#attr::COLUMN.id.is_in(id.to_vec()))
                    .exec(db)
                    .await
                    .map(|x| x.rows_affected)
                    .map_err(Into::into)
            }
        });
    }
    if !contains("count") {
        input.items.push(parse_quote! {
            pub async fn count<C>(
                db: &C,
                cond: actix_cloud_extra::api::Condition,
            ) -> anyhow::Result<u64>
            where
                C: sea_orm::ConnectionTrait,
            {
                Ok(cond.build(#attr::Entity::find()).0.count(db).await?)
            }
        });
    }

    quote! {
        #input
    }
    .into()
}

#[cfg(feature = "seaorm")]
/// Default timestamp generator.
///
/// Automatically generate `created_at` and `updated_at` on create and update.
///
/// # Examples
/// ```ignore
/// pub struct Model {
///     ...
///     pub created_at: DateTime,
///     pub updated_at: DateTime,
/// }
///
/// #[entity_timestamp]
/// impl ActiveModel {}
/// ```
#[proc_macro_attribute]
pub fn entity_timestamp(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut entity = syn::parse_macro_input!(input as syn::ItemImpl);
    entity.items.push(syn::parse_quote!(
        fn entity_timestamp(&self, e: &mut Self, insert: bool) {
            let tm: sea_orm::ActiveValue<DateTime> =
                sea_orm::ActiveValue::set(chrono::Utc::now().naive_utc());
            if insert {
                e.created_at = tm.clone();
                e.updated_at = tm.clone();
            } else {
                e.updated_at = tm.clone();
            }
        }
    ));
    quote! {
        #entity

        impl actix_cloud_extra::entity::DefaultColumnTrait for Column {
            fn get_created_at() -> impl sea_orm::ColumnTrait {
                Self::CreatedAt
            }

            fn get_updated_at() -> impl sea_orm::ColumnTrait {
                Self::UpdatedAt
            }
        }
    }
    .into()
}

#[cfg(feature = "seaorm")]
/// Default id generator.
///
/// Automatically generate `id` on create.
///
/// # Examples
/// ```ignore
/// pub struct Model {
///     id: i64,
///     ...
/// }
///
/// #[entity_id(rand_i64())]
/// impl ActiveModel {}
/// ```
#[proc_macro_attribute]
pub fn entity_id(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as syn::ExprCall);
    let mut entity = syn::parse_macro_input!(input as syn::ItemImpl);
    entity.items.push(syn::parse_quote!(
        fn entity_id(&self, e: &mut Self, insert: bool) {
            if insert && e.id.is_not_set() {
                e.id = sea_orm::ActiveValue::set(#attr);
            }
        }
    ));
    quote! {
        #entity
    }
    .into()
}

#[cfg(feature = "seaorm")]
/// Default entity behavior:
/// - `entity_id`
/// - `entity_timestamp`
///
/// # Examples
/// ```ignore
/// #[entity_id(rand_i64())]
/// #[entity_timestamp]
/// impl ActiveModel {}
///
/// #[entity_behavior]
/// impl ActiveModelBehavior for ActiveModel {}
/// ```
#[proc_macro_attribute]
pub fn entity_behavior(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut entity = syn::parse_macro_input!(input as syn::ItemImpl);

    entity.items.push(syn::parse_quote!(
        async fn before_save<C>(self, _: &C, insert: bool) -> Result<Self, sea_orm::DbErr>
        where
            C: sea_orm::ConnectionTrait,
        {
            let mut new = self.clone();
            self.entity_id(&mut new, insert);
            self.entity_timestamp(&mut new, insert);
            Ok(new)
        }
    ));
    quote! {
        #[async_trait::async_trait]
        #entity
    }
    .into()
}

#[cfg(feature = "seaorm")]
/// Implement `into` for entity to partial entity.
/// The fields should be exactly the same.
///
/// # Examples
/// ```ignore
/// #[partial_entity(users::Model)]
/// #[derive(Serialize)]
/// struct Rsp {
///     pub id: i64,
/// }
///
/// let y = users::Model {
///     id: ...,
///     name: ...,
///     ...
/// };
/// let x: Rsp = y.into();
/// ```
#[proc_macro_attribute]
pub fn partial_entity(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as syn::ExprPath);
    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    let name = &input.ident;
    let mut fields = Vec::new();
    for i in &input.fields {
        let field_name = &i.ident;
        fields.push(quote!(#field_name: self.#field_name,));
    }

    quote! {
        #input
        impl Into<#name> for #attr {
            fn into(self) -> #name {
                #name {
                    #(#fields)*
                }
            }
        }
    }
    .into()
}
