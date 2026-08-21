use abaco::{Evaluator, UnitCategory, UnitRegistry};

fn main() {
    // Expression evaluation
    let eval = Evaluator::new();

    let result = eval.eval("2 + 3 * 4").unwrap();
    println!("2 + 3 * 4 = {result}");

    let result = eval.eval("sqrt(144) + sin(pi / 2)").unwrap();
    println!("sqrt(144) + sin(pi/2) = {result}");

    let result = eval.eval("200 * 15%").unwrap();
    println!("200 * 15% = {result}");

    let result = eval.eval("1.5e3 + 2.5e2").unwrap();
    println!("1.5e3 + 2.5e2 = {result}");

    // Variables
    let mut eval = Evaluator::new();
    eval.set_variable("x", 42.0);
    let result = eval.eval("x ^ 2 + 1").unwrap();
    println!("x=42, x^2 + 1 = {result}");

    // Unit conversion
    let registry = UnitRegistry::new();

    let r = registry.convert(100.0, "celsius", "fahrenheit").unwrap();
    println!("\n{r}");

    let r = registry.convert(5.0, "km", "mi").unwrap();
    println!("{r}");

    let r = registry.convert(1.0, "GB", "GiB").unwrap();
    println!("{r}");

    let r = registry.convert(1.0, "TiB", "GB").unwrap();
    println!("{r}");

    // List available units
    println!("\nData size units:");
    for unit in registry.list_units(UnitCategory::DataSize) {
        println!("  {unit}");
    }
}
