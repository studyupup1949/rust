# winnow, making parsing a breeze

NOTE: This is a fork of [winnow](https://github.com/winnow-rs/winnow) `v0.7.14` with the following changes:  
- Optimised debug rendering with large input streams. Github issue [here](https://github.com/winnow-rs/winnow/issues/871)
- [env_logger](https://github.com/rust-cli/env_logger)-like parser trace filtering. `WINNOW_TRACE=crate::module::function(=children)`
- `trace!` and `trace_name!` macros to automatically create fully qualified parser names when using the `trace` combinator


Example output:  
```
> poe_data_tools::file_parsers::psg::parser::group  | [169, 59, 89, 198, 82, 95
 > winnow::binary::le_f32                           | [169, 59, 89, 198, 82, 95
 < winnow::binary::le_f32                           | +4
 > winnow::binary::le_f32                           | [82, 95, 206, 69, 0, 0, 0
 < winnow::binary::le_f32                           | +4
 > winnow::binary::le_u32                           | [0, 0, 0, 0, 0, 0, 0, 0, 
 < winnow::binary::le_u32                           | +4
 > winnow::binary::le_u32                           | [0, 0, 0, 0, 0, 1, 0, 0, 
 < winnow::binary::le_u32                           | +4
 > winnow::binary::u8                               | [0, 1, 0, 0, 0, 125, 204,
 < winnow::binary::u8                               | +1
 > winnow::binary::length_repeat                    | [1, 0, 0, 0, 125, 204, 0,
  > winnow::binary::le_u32                          | [1, 0, 0, 0, 125, 204, 0,
  < winnow::binary::le_u32                          | +4
  > winnow::combinator::multi::repeat               | [125, 204, 0, 0, 0, 0, 0,
   > poe_data_tools::file_parsers::psg::parser::passive | [125, 204, 0, 0, 0, 0, 0,
    > winnow::binary::le_u32                        | [125, 204, 0, 0, 0, 0, 0,
    < winnow::binary::le_u32                        | +4
    > winnow::binary::le_i32                        | [0, 0, 0, 0, 0, 0, 0, 0, 
    < winnow::binary::le_i32                        | +4
    > winnow::binary::le_u32                        | [0, 0, 0, 0, 0, 0, 0, 0, 
    < winnow::binary::le_u32                        | +4
    > winnow::binary::length_repeat                 | [0, 0, 0, 0, 210, 38, 89,
     > winnow::binary::le_u32                       | [0, 0, 0, 0, 210, 38, 89,
     < winnow::binary::le_u32                       | +4
     > winnow::combinator::multi::repeat            | [210, 38, 89, 198, 143, 1
     < winnow::combinator::multi::repeat            | +0
    < winnow::binary::length_repeat                 | +4
   < poe_data_tools::file_parsers::psg::parser::passive | +16
  < winnow::combinator::multi::repeat               | +16
 < winnow::binary::length_repeat                    | +20
< poe_data_tools::file_parsers::psg::parser::group  | +37
> poe_data_tools::file_parsers::psg::parser::group:1 | [210, 38, 89, 198, 143, 1
```
