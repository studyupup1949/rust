use syn::{parse_macro_input,  Data, DeriveInput, Fields, Field, Type};
use proc_macro2::{TokenStream, TokenTree, Literal};
use quote::quote;

#[proc_macro_derive(ActiveRecord, attributes(active_record))]
pub fn derive_active_record(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let is_ident = |path: &syn::Path, name: &str| path.get_ident().map(|i| i == name) == Some(true);

    let has_tag = |field: &Field, tag: &str| field.attrs.iter().any(|attr| {
        is_ident(attr.path(), "active_record") && 
        attr.parse_args::<syn::Ident>().map(|i| i == tag).unwrap_or(false)
    });

    let name_str: String = input.attrs.iter().find_map(|attr| {
        if is_ident(attr.path(), "active_record") {
            let expr: syn::Expr = attr.parse_args().ok()?;
            if let syn::Expr::Assign(assign) = expr {
                if let syn::Expr::Path(ref left) = *assign.left {
                    if is_ident(&left.path, "rename") {
                        if let syn::Expr::Lit(ref right) = *assign.right {
                            if let syn::Lit::Str(litstr) = &right.lit {
                                return Some(litstr.value());
                            }
                        }
                    }

                }
            }
        }
        None
    }).unwrap_or(name.to_string());

    let all_children: bool = input.attrs.iter().find_map(|attr| {
        if is_ident(attr.path(), "active_record")
        && attr.parse_args::<syn::Ident>().map(|i| i == "children").unwrap_or(false) {
            return Some(true);
        }
        None
    }).unwrap_or(false);

  //let all_serde_children: bool = input.attrs.iter().find_map(|attr| {
  //    if is_ident(attr.path(), "active_record")
  //    && attr.parse_args::<syn::Ident>().map(|i| i == "serde_children").unwrap_or(false) {
  //        return Some(true);
  //    }
  //    None
  //}).unwrap_or(false);


     match input.data {
        Data::Struct(struc) => {
            let mut children: Vec<(usize, TokenTree, Type)> = Vec::new();
            let mut selfs: Vec<(usize, TokenTree, Type)> = Vec::new();
            let named = match struc.fields {
                Fields::Named(named) => {
                    for field in named.named.into_iter() {
                        match has_tag(&field, "child") || all_children {
                            true => {
                                children.push((0, TokenTree::Ident(field.ident.unwrap()), field.ty));
                            },
                            false => selfs.push((0, TokenTree::Ident(field.ident.unwrap()), field.ty))
                        }
                    }
                    true
                },
                Fields::Unnamed(unnamed) => {
                    for (index, field) in unnamed.unnamed.into_iter().enumerate() {
                        match has_tag(&field, "child") || all_children {
                            true => children.push((index, TokenTree::Literal(Literal::usize_unsuffixed(index)), field.ty)),
                            false => selfs.push((index, TokenTree::Literal(Literal::usize_unsuffixed(index)), field.ty))
                        }
                    }
                    false
                },
                Fields::Unit => {panic!("Cannot derive ActiveRecord for a Unit Struct")}
            };
            (children.is_empty() && selfs.is_empty()).then(|| {panic!("Cannot derive ActiveRecord for an Empty Struct")});

            let record_type = TokenStream::from_iter(children.iter().map(|(_, name, ty)| {
                let name_str = name.to_string();
                quote!{(#name_str.to_string(), <#ty>::record_type()),}
            }));

            let get_children_mut = TokenStream::from_iter(children.iter().map(|(_, name, _)| {
                let name_str = name.to_string();
                quote!{(#name_str.to_string(), self.#name.get_record_mut()),}
            }));

            let get_children = TokenStream::from_iter(children.iter().map(|(_, name, _)| {
                let name_str = name.to_string();
                quote!{(#name_str.to_string(), self.#name.get_record_ref()),}
            }));

            let get_raw = TokenStream::from_iter(children.iter().map(|(_, name, _)| {
                let name_str = name.to_string();
                quote!{(#name_str.to_string(), self.#name.get_raw()),}
            }));

            let self_tuple = TokenStream::from_iter(
                selfs.iter().map(|(_, name, _)| quote!{&self.#name, })
            );

            let self_names = selfs.iter().map(|(i, name, _)| {
                let named = syn::Ident::new(&format!("_{name}"), proc_macro2::Span::call_site());
                (i, quote!{#named})
            }).collect::<Vec<_>>();

            let name_tuple = TokenStream::from_iter(
                selfs.iter().map(|(_, name, _)| {
                    let name_str = syn::Ident::new(&format!("_{name}"), proc_macro2::Span::call_site());
                    quote!{#name_str, }
                })
            );

            let type_tuple = TokenStream::from_iter(
                selfs.iter().map(|(_, _, ty)| {
                    quote!{#ty, }
                })
            );

            let set_tuple = TokenStream::from_iter(
                selfs.iter().zip(self_names.iter()).map(|((_, name, _), (_, named))| quote!{self.#name = #named;})
            );

            let construct_tuple_named = TokenStream::from_iter(
                selfs.iter().zip(self_names.iter()).map(|((_, name, _), (_, named))| quote!{#name: #named,})
            );

            let children_builder = children.iter().map(|(i, name, ty)| {
                let name_str = name.to_string();
                (i, quote!{<#ty>::from_raw(map.remove(#name_str).unwrap()),})
            }).collect::<Vec<_>>();


            let mut c_tuple = self_names.iter().map(|(i, named)| (*i, quote!{#named,})).collect::<Vec<_>>();

            let construct_children_named = TokenStream::from_iter(
                children.iter().zip(children_builder.iter()).map(|((_, name, _), (_, c))|
                    quote!{#name: #c}
                )
            );

            c_tuple.extend(children_builder);
            c_tuple.sort_by_key(|a| a.0);
            let construct_children = TokenStream::from_iter(c_tuple.into_iter().map(|a| a.1));

            let constructor = match named {
                true => quote!{#name{#construct_tuple_named #construct_children_named}},
                false => quote!{#name(#construct_children)}
            };

            match (!selfs.is_empty(), !children.is_empty()) {
                (true, true) => proc_macro::TokenStream::from(quote!{
                    impl #impl_generics ActiveRecord for #name #ty_generics #where_clause {
                        fn name() -> String {#name_str.to_string()}

                        fn record_type() -> RecordType {
                            RecordType::Struct(Some(true), std::collections::BTreeMap::from([
                                #record_type
                            ]))
                        }

                        fn get_record_mut(&mut self) -> RecordMut<'_> {RecordMut::Struct(self)}

                        fn get_record_ref(&self) -> RecordRef<'_> {RecordRef::Struct(self)}

                        fn get_children_mut(&mut self) -> std::collections::BTreeMap<String, RecordMut<'_>> {
                            std::collections::BTreeMap::from([
                                #get_children_mut
                            ])
                        }

                        fn get_children(&self) -> std::collections::BTreeMap<String, RecordRef<'_>> {
                            std::collections::BTreeMap::from([
                                #get_children
                            ])
                        }

                        fn set_state(&mut self, _state: &str) {}
                        fn get_state(&self) -> Option<String> {
                            let mut s = std::hash::DefaultHasher::new();
                            std::hash::Hasher::write(&mut s, self.get_self().unwrap().as_bytes());
                            Some(format!("{:#x}", std::hash::Hasher::finish(&s)))
                        }

                        fn get_raw(&self) -> RawRecord {
                            RawRecord::Struct(self.get_self().map(|s| (self.get_state(), s)), std::collections::BTreeMap::from([
                                #get_raw
                            ]))
                        }

                        fn from_raw(raw: RawRecord) -> Self {
                            let (inner, mut map) = raw.r#struct();
                            let st = inner.unwrap().1;
                            println!("st: {}", st);
                            let (#name_tuple): (#type_tuple) = serde_json::from_str(&st).unwrap();
                            #constructor
                        }
                    }

                    impl #impl_generics ActiveStruct for #name #ty_generics #where_clause {
                        fn get_self(&self) -> Option<String> {
                            Some(serde_json::to_string(&(#self_tuple)).unwrap())
                        }

                        fn set_self(&mut self, selfs: String) {
                            let (#name_tuple): (#type_tuple) = serde_json::from_str(&selfs).unwrap();
                            #set_tuple
                        }

                        fn state(&self, other: Option<Option<&str>>) -> std::cmp::Ordering {
                            match other {
                                None => std::cmp::Ordering::Greater,
                                Some(other) if other != self.get_state().as_deref() => std::cmp::Ordering::Greater,
                                _ => std::cmp::Ordering::Equal
                            }
                        }
                    }
                }),
                (true, false) => proc_macro::TokenStream::from(quote!{
                    impl #impl_generics ActiveRecord for #name #ty_generics #where_clause
                        where Self: Serialize + for<'a> Deserialize<'a> {
                        fn name() -> String {#name_str.to_string()}

                        fn record_type() -> RecordType {
                            RecordType::Struct(Some(true), std::collections::BTreeMap::default())
                        }

                        fn get_record_mut(&mut self) -> RecordMut<'_> {RecordMut::Struct(self)}
                        fn get_record_ref(&self) -> RecordRef<'_> {RecordRef::Struct(self)}
                        fn get_children_mut(&mut self) -> std::collections::BTreeMap<String, RecordMut<'_>> {std::collections::BTreeMap::default()}
                        fn get_children(&self) -> std::collections::BTreeMap<String, RecordRef<'_>> {std::collections::BTreeMap::default()}
                        fn set_state(&mut self, _state: &str) {}
                        fn get_state(&self) -> Option<String> {
                            let mut s = std::hash::DefaultHasher::new();
                            std::hash::Hasher::write(&mut s, self.get_self().unwrap().as_bytes());
                            Some(format!("{:#x}", std::hash::Hasher::finish(&s)))
                        }

                        fn get_raw(&self) -> RawRecord {
                            RawRecord::Struct(self.get_self().map(|s| (self.get_state(), s)), std::collections::BTreeMap::default())
                        }

                        fn from_raw(raw: RawRecord) -> Self {
                            serde_json::from_str(&raw.r#struct().0.unwrap().1).unwrap()
                        }
                    }

                    impl #impl_generics ActiveStruct for #name #ty_generics #where_clause
                        where Self: Serialize + for<'a> Deserialize<'a> {
                        fn get_self(&self) -> Option<String> {
                            Some(serde_json::to_string(&self).unwrap())
                        }

                        fn set_self(&mut self, selfs: String) {
                            *self = serde_json::from_str(&selfs).unwrap();
                        }

                        fn state(&self, other: Option<Option<&str>>) -> std::cmp::Ordering {
                            match other {
                                None => std::cmp::Ordering::Greater,
                                Some(other) if other != self.get_state().as_deref() => std::cmp::Ordering::Greater,
                                _ => std::cmp::Ordering::Equal
                            }
                        }
                    }
                }),
                (false, true) => proc_macro::TokenStream::from(quote!{
                    impl #impl_generics ActiveRecord for #name #ty_generics #where_clause {
                        fn name() -> String {#name_str.to_string()}

                        fn record_type() -> RecordType {
                            RecordType::Struct(None, std::collections::BTreeMap::from([
                                #record_type
                            ]))
                        }

                        fn get_record_mut(&mut self) -> RecordMut<'_> {RecordMut::Struct(self)}

                        fn get_record_ref(&self) -> RecordRef<'_> {RecordRef::Struct(self)}

                        fn get_children_mut(&mut self) -> std::collections::BTreeMap<String, RecordMut<'_>> {
                            std::collections::BTreeMap::from([
                                #get_children_mut
                            ])
                        }

                        fn get_children(&self) -> std::collections::BTreeMap<String, RecordRef<'_>> {
                            std::collections::BTreeMap::from([
                                #get_children
                            ])
                        }

                        fn set_state(&mut self, _state: &str) {}
                        fn get_state(&self) -> Option<String> {None}

                        fn get_raw(&self) -> RawRecord {
                            RawRecord::Struct(self.get_self().map(|s| (self.get_state(), s)), std::collections::BTreeMap::from([
                                #get_raw
                            ]))
                        }

                        fn from_raw(raw: RawRecord) -> Self {
                            let (_, mut map) = raw.r#struct();
                            #constructor
                        }
                    }

                    impl #impl_generics ActiveStruct for #name #ty_generics #where_clause {
                        fn get_self(&self) -> Option<String> {None}
                        fn set_self(&mut self, selfs: String) {}
                        fn state(&self, other: Option<Option<&str>>) -> std::cmp::Ordering {
                            std::cmp::Ordering::Equal
                        }
                    }
                }),
                (false, false) => panic!("Cannot derive ActiveRecord for a struct with out at least one child or self field")
            }
        }
        Data::Enum(_) => {panic!("Cannot derive ActiveRecord for a Enum")},
        Data::Union(_) => {panic!("Cannot derive ActiveRecord for a Union")}
    }
}
