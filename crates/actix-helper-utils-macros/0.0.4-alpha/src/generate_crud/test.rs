#![cfg(test)]

use crate::generate_crud::crud_operation::{CrudOperation, OperationType};
use crate::generate_crud::generate_crud_input::GenerateCrudInput;
use proc_macro2::{Ident, Span};
use syn::__private::TokenStream2;
use syn::parse_quote;

#[test]
fn test_parse_generate_crud_input() {
    let input: TokenStream2 = parse_quote! {
        pub struct User {
            pub id: Option<String>,
            pub name: String,
            pub email: String,
        }
        path: "/users",

        create: {
            fn create_user;
            endpoint: create_user_endpoint;
            path: "/users";
            params: {
                #[derive(Serialize, Deserialize, Debug, Clone)]
                struct CreateUserParams {
                    name: String
                }
            }
            query: {
                println!("Creating user");
            }
        }

        read: {
            fn get_user;
            endpoint: get_user_endpoint;
            path: "/users/{id}";
            params: {
                #[derive(Serialize, Deserialize, Debug, Clone)]
                struct GetUserParams {
                    name: String
                }
            }
            query: {
                println!("Reading user");
            }
        }

        update: {
            fn update_user;
            endpoint: update_user_endpoint;
            path: "/users/{id}";
            params: {
                #[derive(Serialize, Deserialize, Debug, Clone)]
                struct UpdateUserParams {
                    name: String
                }
            }
            query: {
                println!("Updating user");
            }
            before: {
                println!("Before update user");
            }
            after: {
                println!("After update user");
            }
        }

        delete: {
            fn delete_user;
            endpoint: delete_user_endpoint;
            path: "/users/{id}";
            params: {
                #[derive(Serialize, Deserialize, Debug, Clone)]
                struct DeleteUserParams {
                    name: String
                }
            }
            query: {
                println!("Deleting user");
            }
        }
    };

    let parsed_input: GenerateCrudInput = syn::parse2(input).unwrap();

    // Check that all CRUD operations exist
    assert!(parsed_input.create_op.is_some());
    assert!(parsed_input.read_op.is_some());
    assert!(parsed_input.update_op.is_some());
    assert!(parsed_input.delete_op.is_some());

    // Validate the create operation
    let create_op = parsed_input.create_op.unwrap();
    assert_eq!(create_op.fn_name.to_string(), "create_user");
    assert_eq!(
        create_op.endpoint_fn_name.to_string(),
        "create_user_endpoint"
    );
    assert_eq!(create_op.path.value(), "/users");
    assert!(create_op.params_struct.is_some());
    assert!(create_op.query_block.is_some());
    assert!(create_op.before_hook.is_none());
    assert!(create_op.after_hook.is_none());

    // Validate the read operation
    let read_op = parsed_input.read_op.unwrap();
    assert_eq!(read_op.fn_name.to_string(), "get_user");
    assert_eq!(read_op.endpoint_fn_name.to_string(), "get_user_endpoint");
    assert_eq!(read_op.path.value(), "/users/{id}");
    assert!(read_op.params_struct.is_some());
    assert!(read_op.query_block.is_some());
    assert!(read_op.before_hook.is_none());
    assert!(read_op.after_hook.is_none());

    // Validate the update operation
    let update_op = parsed_input.update_op.unwrap();
    assert_eq!(update_op.fn_name.to_string(), "update_user");
    assert_eq!(
        update_op.endpoint_fn_name.to_string(),
        "update_user_endpoint"
    );
    assert_eq!(update_op.path.value(), "/users/{id}");
    assert!(update_op.params_struct.is_some());
    assert!(update_op.query_block.is_some());
    assert!(update_op.before_hook.is_some());
    assert!(update_op.after_hook.is_some());

    // Validate the delete operation
    let delete_op = parsed_input.delete_op.unwrap();
    assert_eq!(delete_op.fn_name.to_string(), "delete_user");
    assert_eq!(
        delete_op.endpoint_fn_name.to_string(),
        "delete_user_endpoint"
    );
    assert_eq!(delete_op.path.value(), "/users/{id}");
    assert!(delete_op.params_struct.is_some());
    assert!(delete_op.query_block.is_some());
    assert!(delete_op.before_hook.is_none());
    assert!(delete_op.after_hook.is_none());
}

#[test]
fn test_operation() {
    let input: TokenStream2 = parse_quote! {
        fn update_user;
        endpoint: update_user_endpoint;
        path: "/users/{id}";
        params: {
            #[derive(Serialize, Deserialize, Debug, Clone)]
            struct UpdateUserParams {
                name: String
            }
        }
        query: {
            println!("Updating user");
        }
        before: {
            println!("Before update user");
        }
        after: {
            println!("After update user");
        }
    };

    let parsed_input: CrudOperation = syn::parse2(input).unwrap();

    let struct_ident = Ident::new("User", Span::call_site());

    print!(
        "{}",
        parsed_input.generate("", &struct_ident, OperationType::Update)
    );
}
