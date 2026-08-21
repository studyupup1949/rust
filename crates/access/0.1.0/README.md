# Access

This crate introduces the `Access` trait which has two functions: `get` and `set`.
This allows an implementor to have custom get and set logic around a value.
For example, wrapping a reference in an `Access` provides functionality
to map the getters and setters to another type with the `map` function.

Check the examples folder for more examples.
