use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, Expr, LitStr, Token, Type};

// If you want a helper method for `peek_type`:
trait ParseStreamExt {
    fn peek_type<T: Parse>(&self) -> bool;
}

impl ParseStreamExt for ParseStream<'_> {
    fn peek_type<T: Parse>(&self) -> bool {
        let fork = self.fork();
        fork.parse::<T>().is_ok()
    }
}

pub(crate) struct RequestBody {
    description: Option<LitStr>,
    schema: Option<Type>,
    content: Option<Vec<RequestBodyContent>>,
}

impl Parse for RequestBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fields = Punctuated::<RequestBodyField, Token![,]>::parse_terminated(input)?;

        let mut description = None;
        let mut schema = None;
        let mut content = None;

        for field in fields {
            match field {
                RequestBodyField::Description(lit) => description = Some(lit),
                RequestBodyField::Schema(ty) => schema = Some(ty),
                RequestBodyField::Content(contents) => content = Some(contents),
            }
        }

        Ok(RequestBody {
            description,
            schema,
            content,
        })
    }
}

impl ToTokens for RequestBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // If we have no `content` and do have a `schema` (e.g. request_body = Pet):
        if self.content.is_none() && self.schema.is_some() {
            let schema_ty = self.schema.as_ref().unwrap();
            // produce: request_body = Pet
            tokens.extend(quote! {
                request_body = #schema_ty
            });
            return;
        }

        // Otherwise, produce `request_body(...)`
        // with possible `description`, and a `content(...)` block if `content` is Some.
        let description = &self.description;
        let content_expanded = if let Some(contents) = &self.content {
            // e.g. content((Pet = "application/json", ...), ("text/xml", ...))
            // We expand the list of RequestBodyContent. Each content item
            // will produce something like: (Pet) or (Pet = "application/json", example=..., ...)
            let content_items = contents.iter().map(|item| quote! { (#item) });
            quote! {
                , content #( #content_items ),*

            }
        } else {
            // No multiple content definitions
            quote! {}
        };

        // If a user wrote "request_body(content = String, description = "...", content_type="...")"
        // in your syntax, you likely parse that as `schema=String`, `description=..., content=Some(vec![...])`.
        // We'll produce e.g.:
        // request_body(
        //   description = "some desc",
        //   content((String = "text/xml"))
        // )

        let expanded = quote! {
            request_body(
                description = #description
                #content_expanded
            )
        };

        tokens.extend(expanded);
    }
}

enum RequestBodyField {
    Description(LitStr),
    Schema(Type),
    Content(Vec<RequestBodyContent>),
}

impl Parse for RequestBodyField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "description" => {
                input.parse::<Token![=]>()?;
                let lit = input.parse()?;
                Ok(Self::Description(lit))
            }
            "schema" => {
                input.parse::<Token![=]>()?;
                let ty: Type = input.parse()?;
                Ok(Self::Schema(ty))
            }
            "content" => {
                let content_buf;
                parenthesized!(content_buf in input);

                let contents =
                    Punctuated::<RequestBodyContent, Token![,]>::parse_terminated(&content_buf)?
                        .into_iter()
                        .collect();

                Ok(Self::Content(contents))
            }
            _ => Err(input.error("expected one of `schema` or `content`")),
        }
    }
}

pub(crate) struct RequestBodyContent {
    schema: Option<Type>,
    content_type: Option<LitStr>,
    example: Option<Expr>,
    examples: Option<Vec<Example>>,
}

impl Parse for RequestBodyContent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut schema = None;
        let mut content_type = None;
        let mut example = None;
        let mut examples = None;

        if input.peek_type::<Type>() {
            schema = Some(input.parse()?);

            if input.parse::<Token![=]>().is_ok() {
                content_type = Some(input.parse()?)
            }
        } else if input.peek_type::<LitStr>() {
            content_type = Some(input.parse()?)
        }

        // 2) Possibly parse a comma if user typed one
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        // 3) Now parse named fields like example=... or examples(...)
        while !input.is_empty() {
            // e.g. example=..., or examples(...)
            let field_name: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match field_name.to_string().as_str() {
                "example" => {
                    example = Some(input.parse()?);
                }
                "examples" => {
                    let inside;
                    syn::parenthesized!(inside in input);
                    let items =
                        syn::punctuated::Punctuated::<Example, Token![,]>::parse_terminated(
                            &inside,
                        )?;
                    examples = Some(items.into_iter().collect());
                }
                _ => {
                    return Err(syn::Error::new(
                        field_name.span(),
                        "Expected one of 'example' or 'examples'",
                    ));
                }
            }

            // consume optional trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            schema,
            content_type,
            example,
            examples,
        })
    }
}

impl ToTokens for RequestBodyContent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let schema = &self.schema;
        let content_type = &self.content_type;
        let example = &self.example;
        let examples = &self.examples;

        // We'll build up the inside of the parentheses. For instance:
        //   ( schema = content_type, example = <expr>, examples(...) )
        // or ( schema ), or ( content_type ), etc.

        // Step 1: gather the "schema/content_type" part
        let schema_and_type = match (schema, content_type) {
            (Some(ty), Some(ct)) => {
                // e.g.  (Pet = "application/json", ...)
                quote! { #ty = #ct }
            }
            (Some(ty), None) => {
                // e.g. (Pet)
                quote! { #ty }
            }
            (None, Some(ct)) => {
                // e.g. ("text/xml")
                quote! { #ct }
            }
            (None, None) => {
                // user gave no schema nor content_type => produce an empty block or error
                // We'll produce just (), but you might want to error in your parser instead
                quote! {}
            }
        };

        // Step 2: optional single example
        let example_tok = example.as_ref().map(|ex| {
            quote! { , example = #ex }
        });

        // Step 3: optional multiple examples
        let examples_tok = if let Some(ex_list) = examples {
            // Each Example implements ToTokens => we get something like:
            //   ("John" = (summary = "...", value = ...)), ("Another" = ( ... ))
            //
            // We wrap them into `examples(...)`.
            let items = ex_list.iter().map(|ex| quote! { #ex });
            quote! {
                , examples(
                    #( #items ),*
                )
            }
        } else {
            quote! {}
        };

        // Combine the pieces into a single parenthesized group,
        // e.g. (Pet = "application/json", example = some_expr, examples(...))
        let expanded = quote! {
            #schema_and_type #example_tok #examples_tok
        };

        // The consumer in `RequestBody::to_tokens()` is already putting (#content) around it,
        // so we only produce the *inside*. If you prefer to produce `( ... )` yourself, adjust accordingly.

        tokens.extend(expanded);
    }
}

pub(crate) struct Example {
    name: LitStr,
    summary: Option<LitStr>,
    description: Option<LitStr>,
    value: Option<Expr>,
}

impl Parse for Example {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let outer_content;
        parenthesized!(outer_content in input);

        let name: LitStr = outer_content.parse()?;

        let mut summary = None;
        let mut description = None;
        let mut value = None;

        if outer_content.peek(Token![,]) {
            outer_content.parse::<Token![,]>()?;
        }

        let inner_content;
        parenthesized!(inner_content in outer_content);

        while !inner_content.is_empty() {
            let field_name: Ident = inner_content.parse()?;
            inner_content.parse::<Token![=]>()?;

            match field_name.to_string().as_str() {
                "summary" => {
                    summary = Some(inner_content.parse()?);
                }
                "description" => {
                    description = Some(inner_content.parse()?);
                }
                "value" => {
                    value = Some(inner_content.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(field_name.span(), "unexpected field name"));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            name,
            summary,
            description,
            value,
        })
    }
}

impl ToTokens for Example {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let summary = &self.summary;
        let description = &self.description;
        let value = &self.value;

        // Build the inside of ( ... ), e.g. `summary = "Foo", description = "Bar", value = some_expr`.
        // We can separate them with commas. If any fields are missing, we omit them.
        let summary_tok = summary.as_ref().map(|s| quote! { summary = #s, });
        let description_tok = description.as_ref().map(|d| quote! { description = #d, });
        let value_tok = value.as_ref().map(|v| quote! { value = #v, });

        // If none of them exist, you get () inside.
        // Otherwise you get (summary = "...", description = "...", value = ...).
        let expanded = quote! {
            (#name = (
                #summary_tok
                #description_tok
                #value_tok
            ))
        };

        tokens.extend(expanded);
    }
}
