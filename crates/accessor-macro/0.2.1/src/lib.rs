use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// 为结构体派生 Getter 和 Setter 宏
///
/// 此宏会为带有 `accessor(get)` 属性的字段生成 getter 方法
/// 为带有 `accessor(set)` 属性的字段生成 setter 方法。
///
/// 还支持通过 `accessor(range=[min, max])` 对字段设置值时进行范围检查。
///
/// # 示例
///
/// ```rust
/// use accessor_macro::Accessor;
///
/// #[derive(Accessor, Debug)]
/// struct Person {
///    #[accessor(get, set)]
///    name: String,
///    #[accessor(get, set, range=[0, 200])]
///    age: i32,
/// }
///
/// let mut person = Person {
///     name: "Alice".to_string(),
///     age: 25,
/// };
///
/// assert!(person.get_name().eq("Alice"));
/// person.set_name("Bob".to_string());
/// assert!(person.get_name().eq("Bob"));
/// assert!(person.set_age(18));
/// assert!(person.get_age() == &18);
/// assert!(!person.set_age(-1));
/// assert!(person.get_age() != &-1);
/// assert!(!person.set_age(201));
/// assert!(person.get_age() != &201);
/// ```
#[proc_macro_derive(Accessor, attributes(accessor))]
pub fn accessor_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let fields = if let Data::Struct(data_struct) = &ast.data {
        if let Fields::Named(fields) = &data_struct.fields {
            &fields.named
        } else {
            panic!("Only structs with named fields are supported");
        }
    } else {
        panic!("Accessor can only be derived for structs");
    };

    let getters = fields.iter().filter_map(|field| {
        let field_name = &field.ident;
        let field_ty = &field.ty;
        let attrs = &field.attrs;
        let unaligned = attrs.iter().any(|attr| {
            attr.path.is_ident("accessor") && attr.tokens.to_string().contains("unaligned")
        });

        // 提取字段的文档注释
        let doc_comments: Vec<_> = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("doc"))
            .collect();

        let mut no_ref = false;

        if attrs.iter().any(|attr| {
            let param_string = attr.tokens.to_string();
            if let Some(param_start) = param_string.find("get(") {
                if let Some(param_end) = param_string[param_start..].find(')') {
                    let params = &param_string[param_start + 4..=param_end];
                    if params.contains("no_ref") {
                        no_ref = true;
                    }
                }
            }
            attr.path.is_ident("accessor") && attr.tokens.to_string().contains("get")
        }) {
            let getter_name = format_ident!("get_{}", field_name.as_ref().unwrap());
            if unaligned {
                Some(quote! {
                    #(#doc_comments)*
                    pub fn #getter_name(&self) -> #field_ty {
                        let filed_ptr = std::ptr::addr_of!(self.#field_name);
                        unsafe {
                            filed_ptr.read_unaligned()
                        }
                    }
                })
            } else if no_ref {
                Some(quote! {
                    #(#doc_comments)*
                    pub fn #getter_name(&self) -> #field_ty {
                        self.#field_name
                    }
                })
            } else {
                Some(quote! {
                    #(#doc_comments)*
                    pub fn #getter_name(&self) -> &#field_ty {
                        &self.#field_name
                    }
                })
            }
        } else {
            None
        }
    });

    let setters = fields.iter().filter_map(|field| {
        let field_name = &field.ident;
        let field_ty = &field.ty;
        let attrs = &field.attrs;

        // 提取字段的文档注释
        let doc_comments: Vec<_> = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("doc"))
            .collect();

        if attrs
            .iter()
            .any(|attr| attr.path.is_ident("accessor") && attr.tokens.to_string().contains("set"))
        {
            let setter_name = format_ident!("set_{}", field_name.as_ref().unwrap());
            let range_check = attrs.iter().find_map(|attr| {
                if attr.path.is_ident("accessor") {
                    let tokens = attr.tokens.to_string();

                    if let Some(range_start) = tokens.find("range=[") {
                        let range_str = &tokens[range_start + 7..];
                        let end_index = range_str.find(']').unwrap();
                        let range_values = &range_str[0..end_index];
                        let mut parts = range_values.split(',');
                        let min = parts.next().unwrap().trim();
                        let max = parts.next().unwrap().trim();

                        let min_lit = syn::parse_str::<syn::Expr>(min).ok()?;
                        let max_lit = syn::parse_str::<syn::Expr>(max).ok()?;

                        #[cfg(all(debug_assertions, feature = "debug_panic"))]
                        let out_of_range_handler = Some(quote!{
                            panic!("field '{}' must be between {} and {}", stringify!(#field_name), #min_lit, #max_lit);
                        });
                        #[cfg(any(not(debug_assertions), not(feature = "debug_panic")))]
                        let out_of_range_handler = Some(quote!{
                            return false;
                        });

                        Some(quote! {
                            if value < #min_lit || value > #max_lit {
                                #out_of_range_handler
                            }
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            Some(quote! {
                #(#doc_comments)*
                pub fn #setter_name(&mut self, value: #field_ty) -> bool {
                    #range_check
                    self.#field_name = value;
                    true
                }
            })
        } else {
            None
        }
    });

    let expanded = quote! {
        impl #name {
            #(#getters)*
            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}
