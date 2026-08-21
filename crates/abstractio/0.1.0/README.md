# Abstractio
Abstract IO dimensionality analysis for physics using theory of Avatar Extensions

Brought to you by the [AdvancedResearch](https://advancedresearch.github.io/) community.

[Join us on Discord!](https://discord.gg/JkrhJJRBR2)

```rust
use abstractio::*;

fn main() {
    assert_eq!(density().dim(), [1, 2]);
    assert_eq!(measure_force().dim(), [3, 7]);

    assert_eq!(format!("{:?}", density().to_abstract()), "Bin((Variable, Variable))".to_string());
    assert_eq!(density().to_abstract().dim(), [1, 2]);
}
```

IO dimensionality can be used to determine freedom of degrees in a physical system
and the work required to measure it.

Abstract IO dimensionality analysis is a structure that can be projected down from the algebraic
description of the physical system and used to calculate the IO dimensionality without
loss of information.

### Motivation

The theory of [Avatar Extensions](https://advancedresearch.github.io/avatar-extensions/summary.html)
predicts that there is a level of abstraction where the concrete binary operators do not matter
and where unary operators contract topologically.
In particular, this kind of analysis is important for the semantics of [Avatar Graphs](https://advancedresearch.github.io/avatar-extensions/summary.html#avatar-graphs).

This library shows that this level of abstraction is possible,
using combinatorial properties of "ways to read" and "ways to write".

An algebraic expression describing a physical system or measurement
is analyzed and an IO dimension vector is calculated.
The abstraction level made possible here is shown by projecting the algebraic
expression to an "Abstract IO" data structure.
From this abstract structure, it is possible to compute the IO dimension vector without
loss of information.

Explained in [Path Semantical](https://github.com/advancedresearch/path_semantics) notation:

```text
∴ f[dim_io] <=> f[to_abstract][dim_abstract_io]

∵ dim_io <=> dim_abstract_io . to_abstract
```

This is a tautology in Path Semantics.
However, it is not given that `f[to_abstract]` has a solution,
since Path Semantics has an imaginary inverse operator.

The abstraction is possible if and only if `f[to_abstract]` has a solution.
This property is demonstrated in this library.

### Design

An IO dimension vector is a pair of natural numbers that counts
the number of "ways to read" and "ways to write" in a physical system.

`[<ways to read>, <ways to write>]`

For example, a constant has IO dimensions `[1, 0]`.
The first number is 1 since there is one way to read the value of a constant.
The second number is 0 since it is not possible to change a constant.

Another example: A variable has IO dimensions `[1, 1]`.
The first number is 1 since there is one way to read the value of a variable.
The second number is 1 since there is one way to write a new value to a variable.

A more complex example is `density := mass / volume`.
This system has IO dimensions `[1, 2]`.
The first dimension is 1 since there is one way to read the value of density.
The second dimension is 2 since there are two ways to write a new value to a density,
one way where volume is kept constant and one way where mass is kept constant.
