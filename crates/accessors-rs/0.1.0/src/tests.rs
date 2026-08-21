use quote::quote;

use crate::{accessors_derive_inner, error};

#[test]
fn test_accessors_get() {
    let input = quote! {
        struct MyStruct {
            #[accessors(get)]
            hello_world: String
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn hello_world(&self) -> &String {
                &self.hello_world
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_get_copy() {
    let input = quote! {
        struct MyStruct {
            #[accessors(get_copy)]
            foo: f32
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn foo(&self) -> f32 {
                self.foo
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_get_mut() {
    let input = quote! {
        struct MyStruct {
            #[accessors(get_mut)]
            bar: i32
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn bar_mut(&mut self) -> &mut i32 {
                &mut self.bar
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessorss_set() {
    let input = quote! {
        
        struct MyStruct {
            #[accessors(set)]
            greeting: String
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn set_greeting(&mut self, greeting: String) {
                self.greeting = greeting;
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_with_multiple_on_one_accessors() {
    let input = quote! {
        struct MyStruct {
            #[accessors(get, get_mut, set)]
            hello_world: String
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn hello_world(&self) -> &String {
                &self.hello_world
            }

            pub fn hello_world_mut(&mut self) -> &mut String {
                &mut self.hello_world
            }

            pub fn set_hello_world(&mut self, hello_world: String) {
                self.hello_world = hello_world;
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_with_multiple_on_multiple_accessors() {
    let input = quote! {
        struct MyStruct {
            #[accessors(get)]
            #[accessors(get_mut)]
            #[accessors(set)]
            hello_world: String
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn hello_world(&self) -> &String {
                &self.hello_world
            }

            pub fn hello_world_mut(&mut self) -> &mut String {
                &mut self.hello_world
            }

            pub fn set_hello_world(&mut self, hello_world: String) {
                self.hello_world = hello_world;
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_on_multiple_fields() {
    let input = quote! {
        struct MyStruct {
            #[accessors(get)]
            hello_world: String,
            #[accessors(get_copy)]
            foo: f32,
            #[accessors(get_copy)]
            bar: i32,
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn hello_world(&self) -> &String {
                &self.hello_world
            }

            pub fn foo(&self) -> f32 {
                self.foo
            }

            pub fn bar(&self) -> i32 {
                self.bar
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_get_and_get_copy_are_mutually_exclusive() {
    let input = quote! {
        struct MyStruct {
            #[accessors(get_copy, get)] // get should take over since it's defined after.
            hello_world: String
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn hello_world(&self) -> &String {
                &self.hello_world
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_works_with_generics() {
    let input = quote! {
        struct MyStructGeneric<T: Copy> {
            #[accessors(get)]
            #[accessors(get_mut)]
            #[accessors(set)]
            foo: T,
            #[accessors(get_copy)]
            bar: T
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl<T: Copy> MyStructGeneric<T> {
            pub fn foo(&self) -> &T {
                &self.foo
            }

            pub fn foo_mut(&mut self) -> &mut T {
                &mut self.foo
            }

            pub fn set_foo(&mut self, foo: T) {
                self.foo = foo;
            }

            pub fn bar(&self) -> T {
                self.bar
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_works_with_generics_where() {
    let input = quote! {
        
        struct MyStructGeneric<T> where T: Copy {
            #[accessors(get)]
            #[accessors(get_mut)]
            #[accessors(set)]
            foo: T,
            #[accessors(get_copy)]
            bar: T
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl<T> MyStructGeneric<T> where T: Copy {
            pub fn foo(&self) -> &T {
                &self.foo
            }

            pub fn foo_mut(&mut self) -> &mut T {
                &mut self.foo
            }

            pub fn set_foo(&mut self, foo: T) {
                self.foo = foo;
            }

            pub fn bar(&self) -> T {
                self.bar
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn test_accessors_with_default_accessors() {
    let input = quote! {
        
        #[accessors(get)]
        struct MyStruct {
            hello_world: String,
            #[accessors(get_copy)]
            foo: f32,
            #[accessors(set)]
            bar: u32
        }
    };
    let output = accessors_derive_inner(input).unwrap();
    let expected = quote! {
        impl MyStruct {
            pub fn hello_world(&self) -> &String {
                &self.hello_world
            }

            pub fn foo(&self) -> f32 {
                self.foo
            }

            pub fn bar(&self) -> &u32 {
                &self.bar
            }

            pub fn set_bar(&mut self, bar: u32) {
                self.bar = bar;
            }
        }
    };
    assert_eq!(expected.to_string(), output.to_string());
}

#[test]
fn error_test_accessors_on_union() {
    let input = quote! {
        
        union MyUnion {
            #[accessors(get)]
            hello_world: String
        }
    };
    let output = accessors_derive_inner(input).unwrap_err();
    assert_eq!(error::ACCESSORS_ON_UNION_ERROR_MESSAGE, output.to_string());
}

#[test]
fn error_test_accessors_on_enum() {
    let input = quote! {
        
        enum MyEnum {
            Hello,
            World
        }
    };
    let output = accessors_derive_inner(input).unwrap_err();
    assert_eq!(error::ACCESSORS_ON_ENUM_ERROR_MESSAGE, output.to_string());
}
