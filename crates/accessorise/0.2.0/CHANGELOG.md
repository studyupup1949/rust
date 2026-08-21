# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) (post version 0.2.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version 0.2.0 (15/10/2025)

Added

- Added the trait_macros module and moved all the trait-centric macros into it.

- Added the impl_get_set_val macro.

- Added the impl_trait_get_set_val macro.

- Added the impl_get_ref_mut macro.

- Added the impl_trait_get_ref_mut macro.

- Added the trait_get_ref_mut macro.

- Added the trait_set_val_clone_sig macro.



Changed

- Renamed the macros module to impl_macros.

- Renamed the impl_get macro to impl_get_copy and updated referencing macros accordingly.

- Renamed all macro parameters named $name and $name_type to $field_name and $field_name_type respectively.

- Renamed the impl_trait_get macro to impl_trait_get_copy and updated referencing macros accordingly.

- Renamed the impl_set macro to impl_set_move and updated referencing macros accordingly.

- Renamed the impl_trait_set macro to impl_trait_set_move.

- Renamed the impl_get_set macro to impl_get_copy_set_move.

- Updated the tests module.

- Renamed the trait_get macro to trait_get_val and updated referencing macros accordingly.

- Renamed the trait_set macro to trait_set_val and updated referencing macros accordingly.

- Renamed the trait_get_set macro to trait_get_set_val.

- Renamed the impl_set_move macro to impl_set_val.

- Renamed the impl_trait_set_move macro to impl_trait_set_val.

- Renamed the impl_get_clone macro to impl_get_val.

- Renamed the impl_set_clone macro to impl_set_val_clone.

- Renamed the impl_trait_set_clone macro to impl_trait_set_val_clone.

- Updated the Readme.

- Updated the project description.

- Cleaned up the code a bit.



Removed

- Removed the impl_get_copy and the impl_trait_get_copy macros.

- Removed the impl_get_copy_set_move macro.

- Removed the trait_get_clone macro.



## Version 0.1.0 (17/04/2025)

- Initial release
