# Abhedya KEM

The **Key Encapsulation Mechanism (KEM)** layer for Abhedya.

It implements the underlying Lattice math (Matrix-Vector multiplication) over the specific ring used by Abhedya ($N=768, Q=3329$).

- **Pure Rust**: `no_std` compatible where possible.
- **Constant-Time**: Logic designed to resist timing attacks.

Part of the [Abhedyam](https://github.com/ParamTatva-org/abhedyam) project.
