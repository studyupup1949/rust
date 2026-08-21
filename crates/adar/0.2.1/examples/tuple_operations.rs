use adar::prelude::*;

#[TraitRef]
trait ToStringCapital {
    fn to_string_capital(&self) -> String;
}

impl<T> ToStringCapital for T
where
    T: ToString,
{
    fn to_string_capital(&self) -> String {
        self.to_string()
            .chars()
            .map(|c| c.to_uppercase().next().unwrap())
            .collect()
    }
}

fn main() {
    println!("Homogeneous:");
    let homogeneous = (1, 2, 3, 4);
    for i in homogeneous.iter() {
        println!("\t{i}");
    }

    let mixed = ("String", 24, true, 2.2);
    println!("Mixed -> ToString:");
    // Automatically implemented for all std/core traits
    for i in mixed.iter_trait::<dyn ToString>() {
        println!("\t{}", i.to_string());
    }
    // Needs #[TraitRef] for custom traits
    println!("Mixed -> ToStringCapital:");
    for i in mixed.iter_trait::<dyn ToStringCapital>() {
        println!("\t{}", i.to_string_capital());
    }

    println!("Concat: {:?}", homogeneous.concat(mixed));
    println!("Sum of homogeneous: {:?}", homogeneous.iter().sum::<i32>());
    println!("F32 from mixed: {:?}", mixed.select::<f32>());
    println!("bool from mixed: {:?}", mixed.select::<bool>());
}
