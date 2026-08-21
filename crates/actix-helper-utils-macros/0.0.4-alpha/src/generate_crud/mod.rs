use crate::generate_crud::crud_operation::OperationType;
use crate::generate_crud::generate_crud_input::GenerateCrudInput;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub(crate) mod crud_operation;
pub(crate) mod generate_crud_input;
mod test;

pub(crate) fn generate_crud_internal(input: TokenStream) -> TokenStream1 {
    let input: TokenStream1 = input.into();
    let input = parse_macro_input!(input as GenerateCrudInput);

    let GenerateCrudInput {
        struct_def,
        base_path,
        create_op,
        read_op,
        update_op,
        delete_op,
    } = input;

    let create_tokens = if let Some(create_op) = create_op {
        create_op.generate(&base_path.value(), &struct_def.ident, OperationType::Create)
    } else {
        quote! {}
    };

    let read_tokens = if let Some(read_op) = read_op {
        read_op.generate(&base_path.value(), &struct_def.ident, OperationType::Read)
    } else {
        quote! {}
    };

    let update_tokens = if let Some(update_op) = update_op {
        update_op.generate(&base_path.value(), &struct_def.ident, OperationType::Update)
    } else {
        quote! {}
    };

    let delete_tokens = if let Some(delete_op) = delete_op {
        delete_op.generate(&base_path.value(), &struct_def.ident, OperationType::Delete)
    } else {
        quote! {}
    };

    let tokens = quote! {
        #struct_def

        #create_tokens
        #read_tokens
        #update_tokens
        #delete_tokens
    };

    tokens.into()
}
