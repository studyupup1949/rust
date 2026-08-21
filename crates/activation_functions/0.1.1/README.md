# Activation Functions

### Description:
This is the quellcode of an rust crate called:  `activation_functions`

# Table of Contents
|**Contents**|
|:-|
|[Informations](#some-informations)|
|Documentation [(GitHub)](#documentation) [(docs.rs)](https://docs.rs/activation_functions/0.1.0/activation_functions/)|

# Some informations:
|**Name**|**Value**|
|-|-|
|Language|Rust|
|Programer|SnefDen|
|version|0.1.0|
|last update|10.07.2021|

# Documentation
## Table of Contents
|               |**sigmoid**              |**binary step**      |**tanh**           |**rectified linear unit**|**sigmoid linear unit**|**gaussian**               |
|:--------------|:-----------------------:|:-------------------:|:-----------------:|:-----------------------:|:---------------------:|:-------------------------:|
|[**f32**](#f32)|[f32::sigmoid](#sigmoidx)|[f32::bstep](#bstepx)|[f32::tanh](#tanhx)|[f32::relu](#relux)      |[f32::silu](#silux)    |[f32::gaussian](#gaussianx)|
|[**f64**](#f64)|[f64::sigmoid](#sigmoidx-1)|[f64::bstep](#bstepx-1)|[f64::tanh](#tanhx-1)|[f64::relu](#relux-1)      |[f64::silu](#silux-1)    |[f64::gaussian](#gaussianx-1)|

# f32
## sigmoid(x):
* [Informations](#informations)
* [Example](#example)
* [Implementation](#implementation)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f32;          // float32

```
#### Used inside:
```rust
//      variables
let x:f32;          // parameter
let result:f32;     // return variable

//      functions
std::f32::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f32;     // float32

```
### Example:
```rust
let x:f32   = 0.5;
let answer  = sigmoid(x);

println!("sigmoid({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`sigmoid(0.5) => 0.62245935`
### Implementation:
```rust
pub fn sigmoid(x:f32) -> f32 {
    1 as f32 / (1 as f32 + std::f32::consts::E.powf(-x))
}
```
___
## bstep(x):
* [Informations](#informations-1)
* [Example](#example-1)
* [Implementation](#implementation-1)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f32;          // float32

```
#### Used inside:
```rust
//      variables
let x:f32;          // parameter
let result:f32;     // return variable

//      functions
std::f32::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f32;     // float32

```
### Example:
```rust
let x:f32   = 0.5;
let answer  = bstep(x);

println!("bstep({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`bstep(0.5) => 1.0`
### Implementation:
```rust
pub fn bstep(x:f32) -> f32 {
    if x<0 as f32 {
        0 as f32
    } else {
        1 as f32
    }
}
```
___
## tanh(x):
* [Informations](#informations-2)
* [Example](#example-2)
* [Implementation](#implementation-2)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f32;          // float32

```
#### Used inside:
```rust
//      variables
let x:f32;          // parameter
let result:f32;     // return variable

//      functions
std::f32::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f32;     // float32

```
### Example:
```rust
let x:f32   = 0.5;
let answer  = tanh(x);

println!("tanh({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`tanh(0.5) => 0.46211714`
### Implementation:
```rust
pub fn tanh(x:f32) -> f32 {
    (std::f32::consts::E.powf(x) - std::f32::consts::E.powf(-x)) / (std::f32::consts::E.powf(x) + std::f32::consts::E.powf(-x))
}
```
___
## relu(x):
* [Informations](#informations-3)
* [Example](#example-3)
* [Implementation](#implementation-3)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f32;          // float32

```
#### Used inside:
```rust
//      variables
let x:f32;          // parameter
let result:f32;     // return variable

//      functions
std::f32::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f32;     // float32

```
### Example:
```rust
let x:f32   = 0.5;
let answer  = relu(x);

println!("relu({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`relu(0.5) => 1.0`
### Implementation:
```rust
pub fn relu(x:f32) -> f32 {
    if x<=0 as f32 {
        0 as f32
    } else {
        1 as f32
    }
}
```
___
## silu(x):
* [Informations](#informations-4)
* [Example](#example-4)
* [Implementation](#implementation-4)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f32;          // float32

```
#### Used inside:
```rust
//      variables
let x:f32;          // parameter
let result:f32;     // return variable

//      functions
std::f32::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f32;     // float32

```
### Example:
```rust
let x:f32   = 0.5;
let answer  = silu(x);

println!("silu({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`silu(0.5) => 0.31122968`
### Implementation:
```rust
pub fn silu(x:f32) -> f32 {
    x / (1 as f32 + std::f32::consts::E.powf(-x))
}
```
___
## gaussian(x):
* [Informations](#informations-5)
* [Example](#example-5)
* [Implementation](#implementation-5)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f32;          // float32

```
#### Used inside:
```rust
//      variables
let x:f32;          // parameter
let result:f32;     // return variable

//      functions
std::f32::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f32;     // float32

```
### Example:
```rust
let x:f32   = 0.5;
let answer  = gaussian(x);

println!("gaussian({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`gaussian(0.5) => 0.7788008`
### Implementation:
```rust
pub fn gaussian(x:f32) -> f32 {
    std::f32::consts::E.powf(-(x*x))
}
```
___
# f64
## sigmoid(x):
* [Informations](#informations-6)
* [Example](#example-6)
* [Implementation](#implementation-6)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f64;          // float64

```
#### Used inside:
```rust
//      variables
let x:f64;          // parameter
let result:f64;     // return variable

//      functions
std::f64::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f64;     // float64

```
### Example:
```rust
let x:f64   = 0.5;
let answer  = sigmoid(x);

println!("sigmoid({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`sigmoid(0.5) => 0.62245935`
### Implementation:
```rust
pub fn sigmoid(x:f64) -> f64 {
    1 as f64 / (1 as f64 + std::f64::consts::E.powf(-x))
}
```
___
## bstep(x):
* [Informations](#informations-7)
* [Example](#example-7)
* [Implementation](#implementation-7)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f64;          // float64

```
#### Used inside:
```rust
//      variables
let x:f64;          // parameter
let result:f64;     // return variable

//      functions
std::f64::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f64;     // float64

```
### Example:
```rust
let x:f64   = 0.5;
let answer  = bstep(x);

println!("bstep({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`bstep(0.5) => 1.0`
### Implementation:
```rust
pub fn bstep(x:f64) -> f64 {
    if x<0 as f64 {
        0 as f64
    } else {
        1 as f64
    }
}
```
___
## tanh(x):
* [Informations](#informations-8)
* [Example](#example-8)
* [Implementation](#implementation-8)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f64;          // float64

```
#### Used inside:
```rust
//      variables
let x:f64;          // parameter
let result:f64;     // return variable

//      functions
std::f64::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f64;     // float64

```
### Example:
```rust
let x:f64   = 0.5;
let answer  = tanh(x);

println!("tanh({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`tanh(0.5) => 0.46211714`
### Implementation:
```rust
pub fn tanh(x:f64) -> f64 {
    (std::f64::consts::E.powf(x) - std::f64::consts::E.powf(-x)) / (std::f64::consts::E.powf(x) + std::f64::consts::E.powf(-x))
}
```
___
## relu(x):
* [Informations](#informations-9)
* [Example](#example-9)
* [Implementation](#implementation-9)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f32;          // float32

```
#### Used inside:
```rust
//      variables
let x:f32;          // parameter
let result:f32;     // return variable

//      functions
std::f32::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f32;     // float32

```
### Example:
```rust
let x:f32   = 0.5;
let answer  = relu(x);

println!("relu({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`relu(0.5) => 1.0`
### Implementation:
```rust
pub fn relu(x:f32) -> f32 {
    if x<=0 as f32 {
        0 as f32
    } else {
        1 as f32
    }
}
```
___
## silu(x):
* [Informations](#informations-10)
* [Example](#example-10)
* [Implementation](#implementation-10)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f64;          // float64

```
#### Used inside:
```rust
//      variables
let x:f64;          // parameter
let result:f64;     // return variable

//      functions
std::f64::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f64;     // float64

```
### Example:
```rust
let x:f64   = 0.5;
let answer  = silu(x);

println!("silu({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`silu(0.5) => 0.31122968`
### Implementation:
```rust
pub fn silu(x:f64) -> f64 {
    x / (1 as f64 + std::f64::consts::E.powf(-x))
}
```
___
## gaussian(x):
* [Informations](#informations-11)
* [Example](#example-11)
* [Implementation](#implementation-11)
### Informations:
#### Parameter:
```rust
// variable stands for parameter

let x:f64;          // float64

```
#### Used inside:
```rust
//      variables
let x:f64;          // parameter
let result:f64;     // return variable

//      functions
std::f64::consts::E.powf();

```
#### Return:
```rust
// variable stands for return

let result:f64;     // float64

```
### Example:
```rust
let x:f64   = 0.5;
let answer  = gaussian(x);

println!("gaussian({}) => {}",x,answer);
```
that would print out the answer and the given x-value in this format

`gaussian(0.5) => 0.7788008`
### Implementation:
```rust
pub fn gaussian(x:f64) -> f64 {
    std::f64::consts::E.powf(-(x*x))
}
```
___

