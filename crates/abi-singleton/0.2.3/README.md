# ABI SINGLETION

This is a simple ABI singleton Trait helper crate. When you need a trait that only one implementation can exist, you can use this crate.

## Example

A lib crate define a trait that only one implementation exist, you can use it with out `<T>` like code.

```rust
#[api_trait]
pub trait Cat {
    fn eat(food: Food) -> usize;
}

/// Only one kind of cat, so no need to known type with `<T>`.
pub fn cat_eat(food: Food) -> usize {
    //`CatImpl` is auto generated.
    CatImpl::eat(food)
}
```

A crate implements the trait and use lib funcs.

```rust
struct BlackCat;

// There is only one black cat in the world.
#[api_impl]
impl Cat for BlackCat {
    fn eat(food: Food) -> usize {
        println!("black cat eat one");
        food.count - 1
    }
}

fn main() {
    let food = Food { count: 3 };
    println!("There are {} food", food.count);

    let left = cat_eat(food);

    println!("food left: {}", left);
}
```

## Usage

See `test_project` in this repo for examples.

## Limitations

Only C FFI func support, `self` fields are not supported.
