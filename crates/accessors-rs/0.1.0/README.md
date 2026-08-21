# Accessors Derive Macro

Derive macro generating an impl for accessing the fields of a struct.\
Use `#[accessors(get, get_mut, set)]` to defined which accessors you want to have on a field.

List of `accessors` param.
- `get`: Generate a getter returning a reference.
- `get_copy`: Generate a getter returning a copy. (mutually exclusive with get)
- `get_mut`: Generate a mutable getter returning a mutable reference.
- `set`: Generate a setter.

Using `#[accessors(...)]` on a *field* will generate accessors for this specific field.\
Using `#[accessors(...)]` on a *struct* will generate accessors for all field in the struct.

## Examples
Example using `accessors` on fields only.
```rust
#[derive(Accessors)]
struct MyStruct {
    #[accessors(get_copy, get_mut)]
    #[accessors(set)] // Can use multiple `#[accessors(...)]`
    a: u8,
    #[accessors(get, get_mut, set)]
    b: String,
}
```
Example using `accessors` on struct.

```rust
#[derive(Accessors)]
#[accessors(get, get_mut, set)]
struct MyStruct {
    #[accessors(get_copy)] // `get_copy` will overwrite `get`. 
    a: u8,
    b: String,
}
```
Those two example are equivalent and will generate the same code:
```rust
impl MyStruct {
   pub fn a(&self) -> u8 {
       self.a
   }
   pub fn a_mut(&mut self) -> &mut u8 {
       &mut self.a
   }
   pub fn set_a(&mut self, a: u8) {
       self.a = a;
   }
   pub fn b(&self) -> &String {
       &self.b
   }
   pub fn b_mut(&mut self) -> &mut String {
       &mut self.b
   }
   pub fn set_b(&mut self, b: String) {
       self.b = b;
   }

```