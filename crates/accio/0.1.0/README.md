## accio

__This crate is mostly an experiment, and is not intended for production!__

`accio` retrieves code blocks distributed to multiple sites in the build path,
and puts them together in the call site.

It does this at the compile time; therefore no code execution takes place in
load-time (`ctor`) or runtime (any kind of lazy initialization).

It allows for partial implementations of functions, enums,
struct fields, match statements and more.

There are still many quirks with it; and they are __not likely to be
solved in the future__. Just to name a few;
- Macro execution paths are not checked __AT ALL__,
- Conditional compilation flags are blatantly disregarded,
- Incremental compilation might cause some problems with when the macros are re-evaluated,
- Error messages will point to incorrect places,
- The `accio_emit` macro is only matched by name; and name overlaps are not checked for,
- None of the macros in this crate can be nested,
- There is no way for an `accio_emit` statement to dynamically refer to its own path,
- The source path is simply recursively traversed for `.rs` files; instead 
  of verifying that any of these files are imported somewhere,
- There are many possible errors during parsing that are silently omitted,
- It does not work with documentation examples! (Yes, every example in the docs is `no_run`.)
- I simply didn't test how it would behave with multiple crates.

For most use-cases; I can recommend `inventory`, `linkme`, or `ctor` crates.

For automatically importing all modules in a directory, you can use `automod`.

# Contribution

I'm unlikely to respond to issues in this crate; but if you think
you can make this usable in any way; go for it. So PRs are welcome I guess.

# Use cases

`accio` can be used for collecting statements in a function body as follows:

```no_run
use accio::*;

// some_file.rs
accio_emit! {
    // accio_emit's block must be written
    // as `scope_name { code_block }` pairs.
    // multiple blocks can be listed as follows:
    first_scope {
        val = 1;
    }
    second_scope {
        val += 2;
    }
}

// some_other_file.rs
accio_emit! {
    // the same scope name can have multiple
    // blocks; these are merged in an
    // undetermined order.
    second_scope {
        val += 3;
    }
}

// another_file.rs
let mut val = 0;
assert_eq!(val, 0);

// include the first scope:
// this will evaluate to `val = 1;`
accio!(first_scope);
assert_eq!(val, 1);

// include the second scope:
// this will evaluate to either
// `val += 2; val += 3;` or
// `val += 3; val += 2;`.
accio!(second_scope);
assert_eq!(val, 6);
```

A more meaningful use case is automatically gathering enum variants.
However, due to Rust's grammar rules, the `accio!` macro cannot be
placed inside the braces of an enum or a struct. The following would
not compile:
```compile_fail
use accio::*;

enum SomeEnum { 
    accio!(enum_variants)
}
```

This would yield:

```no_compile
error: expected one of `(`, `,`, `=`, `{`, or `}`, found `!`
 --> src/lib.rs:60:10
  |
5 |     accio!(enum_variants)    
  |          ^ expected one of `(`, `,`, `=`, `{`, or `}`
```

Instead, we can use the `accio_body` attribute macro:

```no_run
use accio::*;

#[accio_body(enum_variants)]
enum SomeEnum {
    // this part MUST be empty!
}

#[accio_body(struct_fields)]
struct SomeStruct {
    /* this part MUST be empty! */
}

#[accio_body(array_elems)]
static SOME_ARRAY: &[i32] = &[];

accio_emit! {
    enum_variants {
        FirstVariant,
    }
    enum_variants {
        SecondVariant(String),
    }
    struct_fields {
        pub name: String,
    }
    array_elems: {
        42,
    }
}
// ...and so on
```

Note that `accio_body` implementation places the code into the
first __empty__ curly brace (`{}`) or square bracket (`[]`) scope.
Therefore the following variants will fail:
```no_run
use accio::*;

#[accio_body(enum_variants)]
enum FailingEnum {
    // there is code within the braces
    AlreadySomeVariant,
}

#[accio_body(struct_fields)]
struct FailingStruct; // no braces!
```
You can still add comments, they do not cause issues.

See `examples/` for more detailed examples.