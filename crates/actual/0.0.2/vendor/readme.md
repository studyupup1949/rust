A crate fr learning purposes only
================================
No special features

```rust
pub use bigdecimal::BigDecimal as BD;
fn main() {
    let math = crate::math::new();
    let tables = math.table();
    let mut m = tables.clone();
    m.auto_generate();
    m.print();
    m.reset();
    m.initialize(BD::from(1), BD::from(0), BD::from(10), BD::from(1))
        .print();
        
}
```
# Edgecases:
## 1: Stepper is zero
IF step is 0, we get a infinite loop. to fix that we ask the user to enter a number because current stepper is invalid. 
### Auto Init (.auto_init)
<br> **SNIPPIT FROM .auto_init() METHOD ON STRUCT 'Tables'** <br>
```rust
loop {
     self.step = get_input("By how much do you want to increment the multiplier? ");
     #[allow(clippy::cmp_owned)]
     if self.step == BD::from(0) {
         // self.stepper_0();
         println!("STEP CANT BE 0");
     } else {
         self.initialized = true;
         break;
     }
}
```

### Manual Init (.init())
<br> **SNIPPIT FROM .init() METHOD ON STRUCT 'Tables'** <br>

```rust
if self.step == 0 {
    loop {
        // self.stepper_0();
        self.step = get_input("Please enter a valid stepper :? PRETYPED ");
        if self.step == 0 {
            continue;
        } else {
            break;
        }
        //this should make status = false or wtvr
    }
}
```


## 2: No generation and ran print
IF we call the method ```.print()``` and we have not generated table which is generated through ```.generate()```<br>
it warns the user
<br> **SNIPPIT FROM .print() METHOD ON STRUCT 'Tables'** <br>

```rust
if self.table_data == Vec::new() {
        println!("table_data not generated. needs fixing generating")
}
```