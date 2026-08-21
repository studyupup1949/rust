use crate::generate_crud::crud_operation::CrudOperation;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, ItemStruct, LitStr, Token};

#[derive(Clone)]
pub(crate) struct GenerateCrudInput {
    pub(crate) struct_def: ItemStruct,
    pub(crate) base_path: LitStr,
    pub(crate) create_op: Option<CrudOperation>,
    pub(crate) read_op: Option<CrudOperation>,
    pub(crate) update_op: Option<CrudOperation>,
    pub(crate) delete_op: Option<CrudOperation>,
}

impl Parse for GenerateCrudInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the struct definition
        let struct_def: ItemStruct = input.parse()?;

        let path_ident = input.parse::<Ident>()?;
        if path_ident != "path" {
            return Err(syn::Error::new_spanned(
                path_ident,
                "Expected 'path' keyword",
            ));
        }
        input.parse::<Token![:]>()?;
        let base_path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // Initialize CRUD operations as None
        let mut create_op = None;
        let mut read_op = None;
        let mut update_op = None;
        let mut delete_op = None;

        // Parse CRUD operations
        while !input.is_empty() {
            // Parse the operation keyword (create, read, update, delete)
            let lookahead = input.lookahead1();
            if lookahead.peek(Ident) {
                let op_ident: Ident = input.parse()?;
                input.parse::<Token![:]>()?;
                let content;
                syn::braced!(content in input);

                let op = content.parse::<CrudOperation>()?;

                match op_ident.to_string().as_str() {
                    "create" => create_op = Some(op),
                    "read" => read_op = Some(op),
                    "update" => update_op = Some(op),
                    "delete" => delete_op = Some(op),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            op_ident,
                            "Expected 'create', 'read', 'update', or 'delete'",
                        ));
                    }
                }
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(GenerateCrudInput {
            struct_def,
            base_path,
            create_op,
            read_op,
            update_op,
            delete_op,
        })
    }
}
