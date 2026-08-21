# Changelog

## [2.5.0](https://github.com/INiNiDS/aam-rs/compare/2.4.0...2.5.0) - 2026-05-03

### Added

- *(splitter)* implement split_aam function for multi-section AAML parsing

## [2.4.0](https://github.com/INiNiDS/aam-rs/compare/2.3.0...2.4.0) - 2026-05-03

### Added

- *(splitter)* add AAML splitting functionality with parsing methods
- *(found_value)* add parsing methods for single values and lists

### Other

- *(aam)* update version to 2.3.0 and add TOML translation support

## [2.3.0](https://github.com/INiNiDS/aam-rs/compare/2.2.0...2.3.0) - 2026-04-28

### Added

- *(translator)* add TOML to AAM translation functionality

## [2.2.0](https://github.com/INiNiDS/aam-rs/compare/2.1.0...2.2.0) - 2026-04-22

### Added

- *(aam)* add Python bindings for aam-rs

## [2.1.0](https://github.com/INiNiDS/aam-rs/compare/2.0.6...2.1.0) - 2026-04-21

### Added

- *(aam)* improve error handling in AamValue display implementation

### Fixed

- *(aam)* improve error handling in AamValue display implementation

## [2.0.6](https://github.com/INiNiDS/aam-rs/compare/2.0.5...2.0.6) - 2026-04-21

### Other

- *(release)* improve release workflow script formatting
- *(release)* update release configuration and versioning

## [2.0.5](https://github.com/INiNiDS/aam-rs/compare/v2.0.4...v2.0.5) - 2026-04-21

### Fixed

- remove unused files from git
- *(release)* update Rust version and installation method for dependencies

### Other

- *(release)* update cargo-binstall installation command
- *(release)* update release workflow to include cargo bin in PATH
- *(release)* now release please for non-Rust package versions and release-plz for Rust Packages.
- *(release)* update versioning comments and module names

## [2.0.4](https://github.com/INiNiDS/aam-rs/compare/aam-rs-v2.0.3...aam-rs-v2.0.4) (2026-04-17)


### Bug Fixes

* **aam:** update include path for aam.h in aam.go ([f74d172](https://github.com/INiNiDS/aam-rs/commit/f74d172b5aaf52abafb9182edeb84e2504df7ba0))
* **aam:** update include path for aam.h in aam.go ([b59c550](https://github.com/INiNiDS/aam-rs/commit/b59c550b1324ef050904d7a141c14166d2c97293))
* **aam:** update include path for aam.h in aam.go ([d5f05f6](https://github.com/INiNiDS/aam-rs/commit/d5f05f60948eb525fbd82d311d916ae9c6a71faf))
* **aam:** update include paths and linker flags in aam.go ([706bc05](https://github.com/INiNiDS/aam-rs/commit/706bc056a232afb2f62a8d786af19410630963e6))
* **bindings:** update aam-rs path references in Cargo.toml and aam.go ([611a29e](https://github.com/INiNiDS/aam-rs/commit/611a29e9f3688b0c41725e44ed5bc1618cc394aa))
* Fixed bindings directory ([8bb9e0c](https://github.com/INiNiDS/aam-rs/commit/8bb9e0c36bbc1bfd3af61b02b577cfe9d571608b))
* **formatter:** enhance formatting capabilities for AAM documents ([1f8f9c4](https://github.com/INiNiDS/aam-rs/commit/1f8f9c4892fdb028749f23922a77558cc6dcd741))

## [2.0.3](https://github.com/INiNiDS/aam-rs/compare/aam-rs-v2.0.2...aam-rs-v2.0.3) (2026-04-12)


### Bug Fixes

* enhance schema validation = ([1b1cba9](https://github.com/INiNiDS/aam-rs/commit/1b1cba9575859a1e69ce12be7eabd719edfe548d))

## [2.0.2](https://github.com/INiNiDS/aam-rs/compare/aam-rs-v2.0.1...aam-rs-v2.0.2) (2026-04-08)


### Bug Fixes

* Fixed schema validation ([47d8fad](https://github.com/INiNiDS/aam-rs/commit/47d8fadd1d6abbed1c541396540ea9916977b7d5))
* update migration guide, build.yml, and README for clarity and accuracy ([727ad5c](https://github.com/INiNiDS/aam-rs/commit/727ad5c3fc8c5e9ef4887d1878884c75ae74cb90))

## [2.0.1](https://github.com/INiNiDS/aam-rs/compare/aam-rs-v2.0.0...aam-rs-v2.0.1) (2026-04-03)


### Bug Fixes

* Fixed paths in release.yml ([907be1a](https://github.com/INiNiDS/aam-rs/commit/907be1a9e70416a0bde5f68ac87412eaaca7b663))
* fixed versions ([64d3190](https://github.com/INiNiDS/aam-rs/commit/64d31908536498eb20a2b4313c9e0274e91d3e4e))
* update README and release-please config for version 2.0.0 ([5d53957](https://github.com/INiNiDS/aam-rs/commit/5d539577b2b63c9c1fc4eb6841c9e55091c541d8))

## [2.0.0](https://github.com/INiNiDS/aam-rs/compare/aam-rs-v1.4.0...aam-rs-v2.0.0) (2026-04-02)


### ⚠ BREAKING CHANGES

* **pipeline:** AAM is not AAML compatible, now for make commands and other - use Pipeline modification
* All standard errors have been replaced by custom errors.

### Features

* 1: Added First AAM logic and FoundValue struct ([1fc446c](https://github.com/INiNiDS/aam-rs/commit/1fc446c0b4f4549c8908cbc66b6c33135ae6912d))
* 3: Added find_deep function and tests for this ([259da33](https://github.com/INiNiDS/aam-rs/commit/259da3378b2f656721a5d59c2964ddf6ab85803b))
* 4: Adding comments ignoring ([0ac35c6](https://github.com/INiNiDS/aam-rs/commit/0ac35c6d3da07d73c2c261656520db6af78dce4d))
* 5: Added error module ([5c595a8](https://github.com/INiNiDS/aam-rs/commit/5c595a847f6b415d06d47070dc54a0a2c71f7a5b))
* 6: AAM Builder for building aam file from rust ([3945750](https://github.com/INiNiDS/aam-rs/commit/3945750264b825cb2189a20b4ba99e503312072e))
* 6: Improved parser for normal using quotes and comments. And added a tests for this ([d2c6f82](https://github.com/INiNiDS/aam-rs/commit/d2c6f82d48ae20d61d84a83cfdc0226c53ad2968))
* 7: Added imports and merging ([95988da](https://github.com/INiNiDS/aam-rs/commit/95988da5db5ca69cdde1afea004b06dea48791e1))
* add AamDocument class for JNI bindings and update .gitignore ([57a4a52](https://github.com/INiNiDS/aam-rs/commit/57a4a5205f38bf4c9b210ce4d8e21b87b4c410ae))
* Add BTreeMap import for enhanced data structure support ([79130bd](https://github.com/INiNiDS/aam-rs/commit/79130bdc74e24a7eda250304e76466187b610dd8))
* add C FFI bindings support and gh actions ([31b41a7](https://github.com/INiNiDS/aam-rs/commit/31b41a707461e86a47c535dcb33b259d0ff68655))
* Add C# bindings for aam-rs with support for AAM parsing and configuration management ([16cf956](https://github.com/INiNiDS/aam-rs/commit/16cf95617741896164142acfc45aa8f0d273f45c))
* Add license files and generate CREDITS.html for better compliance ([9d240eb](https://github.com/INiNiDS/aam-rs/commit/9d240eb9e06c5685bdb3c384aad7c582f10a3691))
* Add Node.js bindings for aam-rs with platform-specific packages and build configuration ([12a8073](https://github.com/INiNiDS/aam-rs/commit/12a8073e2a8bf687f543d4f9775d300830c89ec3))
* Add PHP bindings for aam_rs with parsing functionality and smoke tests ([d724e02](https://github.com/INiNiDS/aam-rs/commit/d724e02f0179dab492390274becde3bc886099c4))
* Add PHP package publishing workflow to mirror repository and update artifact handling ([4b87160](https://github.com/INiNiDS/aam-rs/commit/4b87160be2ec9f4f84e78b8fbc0b9cedcfa72cad))
* Add Python bindings and update CI workflow for packaging ([d5035e6](https://github.com/INiNiDS/aam-rs/commit/d5035e67dae6956bbe4f078a5c354e4cf9920778))
* Add Ruby bindings for aam_rs with parsing functionality and tests ([a05f98b](https://github.com/INiNiDS/aam-rs/commit/a05f98bf521b316dad1e091bd32cd7416c29fc21))
* Add serde support ([0bb0918](https://github.com/INiNiDS/aam-rs/commit/0bb0918639231ab0c5365f002240a92353a60330))
* Add WebAssembly bindings for aam_rs with build, test, and release workflows ([14b791a](https://github.com/INiNiDS/aam-rs/commit/14b791aa0678e31417d0dae64248121e0431f655))
* Added better support for C, Python and Java ([e8a7bfe](https://github.com/INiNiDS/aam-rs/commit/e8a7bfe67d3e5b48fa3c8cc8cb8f62bb92fc65e5))
* Added C support ([57b8f92](https://github.com/INiNiDS/aam-rs/commit/57b8f927f10b103d008a2df39673f9bddeea96df))
* Added Go support ([1798cd6](https://github.com/INiNiDS/aam-rs/commit/1798cd637f6efc62065d1d597822b9936b20907f))
* Added schema-in-schema, new examples and list ([98fb857](https://github.com/INiNiDS/aam-rs/commit/98fb857517795e43498d329a1d51d4bf2d1320b5))
* Csharp console ([79f2694](https://github.com/INiNiDS/aam-rs/commit/79f269495054cf15ce388ba871d4e7b88b597ca0))
* Enhance schema validation and improve parser error handling ([3b2a734](https://github.com/INiNiDS/aam-rs/commit/3b2a734c2eb9ea12ab030d15bf52ed82e548eea2))
* Implemented new AAM in other packages ([0393301](https://github.com/INiNiDS/aam-rs/commit/039330143b4f5fef5a75fa7d2178002e196f8629))
* Implementend derive and schemas ([42e8407](https://github.com/INiNiDS/aam-rs/commit/42e8407802bcc6b57083fae17737e5cd6c36a31c))
* Improve error handling and validation in AAM parser and add comprehensive tests ([ae9f10b](https://github.com/INiNiDS/aam-rs/commit/ae9f10ba8a00afbf23845790b68bd579f09e87b6))
* Introduce AAMBuilder and SchemaField for building AAM content programmatically ([b3aedb2](https://github.com/INiNiDS/aam-rs/commit/b3aedb2c72379895d9a061e15857cffffd307fcc))
* Optimization and types ([3d6c6e7](https://github.com/INiNiDS/aam-rs/commit/3d6c6e780400048437203931cec98ca336fea99c))
* **pipeline:** complete zero-copy architecture and fix lifetimes ([1a4a4af](https://github.com/INiNiDS/aam-rs/commit/1a4a4af82e91a2820b5631db63b7523977c792dd))
* **pipeline:** refactor AAML to AAM and enhance parsing logic. ([eb88846](https://github.com/INiNiDS/aam-rs/commit/eb88846a37f2035223d7c7b402434dfe7c81c75e))
* Refactor tests to use 'get' and 'reverse_search' methods, improving clarity and functionality ([1273607](https://github.com/INiNiDS/aam-rs/commit/1273607ec556ebd07bf66ddeab41f868c6e829f0))
* Refactor tests to use AAM instead of AAML, enhancing coverage and error handling ([a67f1c2](https://github.com/INiNiDS/aam-rs/commit/a67f1c2bdc1b7fea3fcee8a5cb930af477ba534b))
* Removes legacy bindings and adds comprehensive tests ([10edb3b](https://github.com/INiNiDS/aam-rs/commit/10edb3b8152f646f3b5a8a4a3b6020dab8aaf656))
* Update AAM API to use new handle structure and enhance functionality with additional methods ([97efd89](https://github.com/INiNiDS/aam-rs/commit/97efd89f9d4d4954db653f5b9be786835b5177a4))
* Update build process for Go bindings and add Maven Central upload ([ebb09a3](https://github.com/INiNiDS/aam-rs/commit/ebb09a39e06c4c5e130673fd84c7b35f1a543e0f))
* Update package names and improve Ruby gem publishing workflow ([4d4643d](https://github.com/INiNiDS/aam-rs/commit/4d4643d63b1ec6859bd017cd023eb4d24724e745))
* Upgrade target framework to net10.0 and enhance AamDocument with detailed XML documentation ([bcfce04](https://github.com/INiNiDS/aam-rs/commit/bcfce04c58dd3f4566fc3fdd5ffe787decf1ebf0))


### Bug Fixes

* .gitignore fix ([1731df2](https://github.com/INiNiDS/aam-rs/commit/1731df228250449c9b7db387921e9564b8345e1a))
* 2: Fixed tests and optimized code ([cfcacc1](https://github.com/INiNiDS/aam-rs/commit/cfcacc1c24d75f7834755fa46d8956a875db0306))
* 3: English in test errors. ([f94981b](https://github.com/INiNiDS/aam-rs/commit/f94981b67c2232ee00c4b6aac2d79daf5c23e942))
* Add C examples demonstrating aam-rs API usage and configuration loading ([258af0b](https://github.com/INiNiDS/aam-rs/commit/258af0b5258e0475c9f3eb08f50b490ecfbd6879))
* Added deny.toml file for cargo deny ([064c05e](https://github.com/INiNiDS/aam-rs/commit/064c05e2c0c301d760ecec7d7e72000298df3b19))
* Correct typo in error handling for AAML parsing ([ce43894](https://github.com/INiNiDS/aam-rs/commit/ce43894077fd469d222d1f5b090c449feb372e6b))
* Deleted unnecessary file ([ac64d9a](https://github.com/INiNiDS/aam-rs/commit/ac64d9a5cbb798485aaaf157e59fde83325b4acf))
* Examples ([180111f](https://github.com/INiNiDS/aam-rs/commit/180111f55661482c5173a621fe5928b3ea7eec0c))
* Fixed [@type](https://github.com/type) in derive_base.aam ([5d956d3](https://github.com/INiNiDS/aam-rs/commit/5d956d37494eb35430983de35bded19504c45ba6))
* Fixed Build for java ([999a4ba](https://github.com/INiNiDS/aam-rs/commit/999a4badc942a9cb230ff7c64ecc7c23a88a4b2d))
* Fixed deny.toml ([3eb10d3](https://github.com/INiNiDS/aam-rs/commit/3eb10d30a99877ec1c1522f7967330e426a20807))
* Fixed deny.toml ([fa699a6](https://github.com/INiNiDS/aam-rs/commit/fa699a618a2c72a80c61013eadebaf40b8006467))
* Fixed deny.toml ([980ea09](https://github.com/INiNiDS/aam-rs/commit/980ea090c417186057459d8400eb7ac28eafad49))
* Fixed deny.toml ([e94c2e9](https://github.com/INiNiDS/aam-rs/commit/e94c2e9705ec91bf609d17d822b5551a998bc254))
* Fixed derive for validation of types ([304ac70](https://github.com/INiNiDS/aam-rs/commit/304ac70899975ca4a6fbd55082fbadb8ec118a3f))
* Fixed Formatting ([2c54f7b](https://github.com/INiNiDS/aam-rs/commit/2c54f7bf9ea6d0243d3b068b8c2b33f7fde6d5cc))
* Fixed gradle configuration ([dddf020](https://github.com/INiNiDS/aam-rs/commit/dddf020b82383f43341529bb4336e938e485fa0f))
* fixed mismatched types when use ahash ([d6b61b9](https://github.com/INiNiDS/aam-rs/commit/d6b61b9192cbd7b95be2c456ce51df375936817a))
* Fixed release-please-config.json ([b4eb3be](https://github.com/INiNiDS/aam-rs/commit/b4eb3bed0ce88bc88c9cda5f2e951b8a2269ddba))
* fixed that parser cannot parse with error some normal files ([c6ebf66](https://github.com/INiNiDS/aam-rs/commit/c6ebf66a863e9216a7862d551462f74a3ecf1bd5))
* Fixed using schema as type in resolve_builtin() ([5d158e7](https://github.com/INiNiDS/aam-rs/commit/5d158e7daa6d543b6f6e877903c534492bb282c2))
* Fixed validation all fields ([58dc757](https://github.com/INiNiDS/aam-rs/commit/58dc757a784208c75ec5d61bb01a97552b54585e))
* Now derive automatically import all types that used in schema ([f26a64b](https://github.com/INiNiDS/aam-rs/commit/f26a64b468e697b2774d48690eb71413c754f730))
* Refactor AAM parsing and loading methods for improved clarity and efficiency ([8c464ca](https://github.com/INiNiDS/aam-rs/commit/8c464cae08a71f9f4a780aa2d156cfd3ca61d859))
* Remove unnecessary NuGet API key check in C# package publishing condition ([57feab7](https://github.com/INiNiDS/aam-rs/commit/57feab7e0cba9bd6bbdc56af9cc3816bf5b131fd))
* Run Release please ([2eb686a](https://github.com/INiNiDS/aam-rs/commit/2eb686ae94a9deb511ce46bf44db48892bce8ce1))
* Set LD_LIBRARY_PATH for Go bindings testing ([00d8f86](https://github.com/INiNiDS/aam-rs/commit/00d8f8656ee1b5d071a1205d8701bfaffed37fd1))
* Set LD_LIBRARY_PATH for Go bindings testing ([f0ecdd9](https://github.com/INiNiDS/aam-rs/commit/f0ecdd9fb7edd50d5a4c8fc63c51b8dd60e3a9e3))
* Simplify condition for publishing C# package by removing NuGet API key check ([cd143b5](https://github.com/INiNiDS/aam-rs/commit/cd143b5b043b39dba64120c8f729557bfd87f6ae))
* Simplify RubyAam and RubyAamBuilder implementations and improve method organization ([cd7aea3](https://github.com/INiNiDS/aam-rs/commit/cd7aea3eb106e199c2951cd01d06908fd83a171d))
* Update about.toml to exclude 'aam-rs' package from processing ([0197da8](https://github.com/INiNiDS/aam-rs/commit/0197da81cfc15f5de3d522b07a3b74bed4ac29d6))
* Update build and release workflows to set git branch and enable CLI fetch ([cad9fbb](https://github.com/INiNiDS/aam-rs/commit/cad9fbb9e78933532eeed5180677daf1fd20d6fc))
* update build commands and dependencies for improved compatibility ([6491656](https://github.com/INiNiDS/aam-rs/commit/6491656fde9131bc30995414d32c90c1e22fc314))
* Update build.gradle.kts to use uri() for staging repo URL and improve build.yml for OS-specific binary archiving ([ee2ae00](https://github.com/INiNiDS/aam-rs/commit/ee2ae00cce458a4eded6c61e5ed9296de76a5627))
* Update magnus dependency URL and improve test module imports ([00e5360](https://github.com/INiNiDS/aam-rs/commit/00e536035d2fb62bfce8702e4a89265f76838bc1))
* Update publish conditions for Java artifacts and remove npm publishing steps from build.yml ([97f0a74](https://github.com/INiNiDS/aam-rs/commit/97f0a74ede34fedc130f9cf00dd56e93af16c3c0))
* Update references from AAML to AAM and improve method names for consistency ([43cd8ac](https://github.com/INiNiDS/aam-rs/commit/43cd8ac115c2dc92c4429486064190a1ef64632f))
* Update RubyAam and RubyAamBuilder to use RArray for field parameters and improve error handling ([ccc0739](https://github.com/INiNiDS/aam-rs/commit/ccc07390da9cb44dd518ff16d9f1e71306494538))
* Update working directory for C# tests in release workflow ([f803087](https://github.com/INiNiDS/aam-rs/commit/f803087ee43ab94f7a34860bc3edde9fa075be85))
* Upgrade .NET version to 10.0 and update project references in build and release workflows ([4fd1bc9](https://github.com/INiNiDS/aam-rs/commit/4fd1bc969a4ec9641922979995308f98d0e094b1))


### Code Refactoring

* Enhance error handling and diagnostics across multiple modules ([28acb2d](https://github.com/INiNiDS/aam-rs/commit/28acb2d7c08a368ab3e8e996dddf76743a54afe5))

## [1.4.0](https://github.com/INiNiDS/aam-rs/compare/v1.3.3...v1.4.0) (2026-03-12)


### Features

* Add Python bindings and update CI workflow for packaging ([d5035e6](https://github.com/INiNiDS/aam-rs/commit/d5035e67dae6956bbe4f078a5c354e4cf9920778))


### Bug Fixes

* Fixed deny.toml ([3eb10d3](https://github.com/INiNiDS/aam-rs/commit/3eb10d30a99877ec1c1522f7967330e426a20807))

## [1.3.3](https://github.com/INiNiDS/aam-rs/compare/v1.3.2...v1.3.3) (2026-03-11)


### Bug Fixes

* Add C examples demonstrating aam-rs API usage and configuration loading ([258af0b](https://github.com/INiNiDS/aam-rs/commit/258af0b5258e0475c9f3eb08f50b490ecfbd6879))

## [1.3.2](https://github.com/INiNiDS/aam-rs/compare/v1.3.1...v1.3.2) (2026-03-11)


### Bug Fixes

* Deleted unnecessary file ([ac64d9a](https://github.com/INiNiDS/aam-rs/commit/ac64d9a5cbb798485aaaf157e59fde83325b4acf))

## [1.3.1](https://github.com/INiNiDS/aam-rs/compare/v1.3.0...v1.3.1) (2026-03-11)


### Bug Fixes

* Run Release please ([2eb686a](https://github.com/INiNiDS/aam-rs/commit/2eb686ae94a9deb511ce46bf44db48892bce8ce1))

## [1.3.0](https://github.com/INiNiDS/aam-rs/compare/v1.2.4...v1.3.0) (2026-03-11)


### Features

* Added C support ([57b8f92](https://github.com/INiNiDS/aam-rs/commit/57b8f927f10b103d008a2df39673f9bddeea96df))
