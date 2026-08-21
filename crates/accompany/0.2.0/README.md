# accompany
`with`-like macro for Rust, which helps narrow down lifetime of a variable.

## Usages
Installation: add `accompany = "0.2.0"` to your `Cargo.toml`.

Examples:
```rust
use accompany::bound;
let i = bound!{
    with j = 1 => {
        let m = j + 1;
        m
    }
};
```
This will be translated to 
```rust
let i ={
    let j = 1;
    let m = j + 1;
    m
};
```

Or simply
```rust
let i = bound!{
    with j = 1 => j + 1
};
```

And you can do it `with` multiple variables of importance.
```rust
let i = bound!{
    with j = 1, k =1 => {
        let m = j + k;
        m
    }
};
```

Also, destruction of structs and tuples are supported.
```rust
let tup = (1,2);
let i = bound!{
    with (i,j) = tuple =>{
        let m= i+j;
        m
    }
};

pub struct A{
    pub field: u8,
}

let a = A{ field : 0};
let i = bound!{
    with A {field: mut i} = a => {
        i += 1;
        let m = i + 1;
        m
    }
};
```

This is nothing fancy, but it helps to keep track of and to limit the lifetime of a variable of importance.

It is especially useful when `rustc` fails to narrow down the lifetime of a key variable and thus throws a compile error.

For example:
```rust
struct B {
    pub field: u8,
}

struct C<'a> {
    pub some_mut_ref: &'a mut u8,
}

impl Drop for C<'_> {
    fn drop(&mut self) {
        *self.some_mut_ref += 1;
    }
}

impl B {
    pub fn return_c<'a>(&'a mut self) -> C<'a> {
        C { some_mut_ref: &mut self.field}
    }
}


fn main() {
    let mut b = B { field: 0 };
    let mut c : C = b.return_c();
    println!("{}", c.some_mut_ref);
    // expect `c` is dropped here
    let ref_to_b = &b;
    println!("{}", ref_to_b.field);
    // actually `c` is dropped here, thus rustc gives a compile error
}
```

Now with `bound!{}`, we can do
```rust
fn main() {
    let mut b = B { field: 0 };
    bound!{
        with mut c = b.return_c() => {
            println!("{}", c.some_mut_ref);
        }
    } // `c` is dropped right here
    let ref_to_b = &b;
    println!("{}", ref_to_b.field);
}
```
which is better than the below which does NOT emphasize the variable of importance.
```rust
fn main() {
    let mut b = B { field: 0 };
    {
        let mut c = b.return_c();
        println!("{}", c.some_mut_ref);
    } // `c` is dropped right here
    let ref_to_b = &b;
    println!("{}", ref_to_b.field);
}
```
