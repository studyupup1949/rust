# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.1](https://git.sr.ht/~onemoresuza/aba/refs/0.6.1) - 2023-10-24

### Bug Fixes

- Set `From` as default header for parse ([7f56fdf](https://git.sr.ht/~onemoresuza/aba/commit/7f56fdf099331c2f10b5c698b16bf38e01e21f76))

### Build

- Remove need to set archive_name inside of nix ([c52d756](https://git.sr.ht/~onemoresuza/aba/commit/c52d756e3339ffe7c0d32b03431044c0175ff5bf))
- Capitalize `just` variables ([96aca4e](https://git.sr.ht/~onemoresuza/aba/commit/96aca4e87461237de99f2b43537aeabd2e747b3a))
- Create a `dist` package ([82b61ea](https://git.sr.ht/~onemoresuza/aba/commit/82b61ea0909d3c0f15c219c605127311918a3887))

## [0.6.0](https://git.sr.ht/~onemoresuza/aba/refs/0.6.0) - 2023-10-23

### Build

- Remove symbol stripping ([b5c13e2](https://git.sr.ht/~onemoresuza/aba/commit/b5c13e2c8e8fc02889ea9d67665ecbf5849ec68a))

## [0.5.1](https://git.sr.ht/~onemoresuza/aba/refs/0.5.1) - 2023-10-23

### Build

- Push musl artifacts of musl builds only on new release ([19c5cf9](https://git.sr.ht/~onemoresuza/aba/commit/19c5cf99a13eb61b6637857f11ff50889a87c291))
- Fix indentation of task ([054d679](https://git.sr.ht/~onemoresuza/aba/commit/054d6790c72a6b908998bf2ef254fa83cd1be0cc))
- Change glob of `dist-bin` recipe ([b91339e](https://git.sr.ht/~onemoresuza/aba/commit/b91339e89e27fc992973d1261659510db9a5124b))

### Documentation

- Set correct file format on manpage 5 ([1b1543a](https://git.sr.ht/~onemoresuza/aba/commit/1b1543a45b9344753652067efea2b16b6363fc60))

### Miscellaneous Tasks

- Add some keys to Cargo.toml package table ([a41b14e](https://git.sr.ht/~onemoresuza/aba/commit/a41b14eb3345045ea59e8b95fc23dedc34bf7400))

## [0.5.0](https://git.sr.ht/~onemoresuza/aba/refs/0.5.0) - 2023-10-23

### Build

- Strip binaries with release profile ([e92c7aa](https://git.sr.ht/~onemoresuza/aba/commit/e92c7aacc3a7a0c28eef90fc7d17063b19012ad2))

### Miscellaneous Tasks

- Add mento to pre-built binaries on README ([9196688](https://git.sr.ht/~onemoresuza/aba/commit/9196688bca5e0151ad4597cc4e87fc78b2a86c27))
- Fix pre-built binary install ([8032cc2](https://git.sr.ht/~onemoresuza/aba/commit/8032cc28c7525780f7984880ac3e4953a947803b))
- Fix pre-built binary install ([458c5e4](https://git.sr.ht/~onemoresuza/aba/commit/458c5e47a7ee3122d0ac0ff634cd722710fa535e))
- Fix instructions and links in README ([33dbf85](https://git.sr.ht/~onemoresuza/aba/commit/33dbf85bdbc9e36fa0cd3ad91410e1e2bde21d9a))
- Fix instructions and links in README ([9b02fee](https://git.sr.ht/~onemoresuza/aba/commit/9b02feeedb8eada65655bd47ec8cceb12f729756))

## [0.4.0](https://git.sr.ht/~onemoresuza/aba/refs/0.4.0) - 2023-10-23

### Build

- Remove unneeded `mkdir` from install recipe ([9224b98](https://git.sr.ht/~onemoresuza/aba/commit/9224b98be00d4111804827e67e0845f5d1fc5b69))
- Change `bin` recipe to `build` recipe ([18b5dba](https://git.sr.ht/~onemoresuza/aba/commit/18b5dbac1af9764e154c5ba41bead04f1947fe1a))
- Add an attempt at static binaries ([4bf9476](https://git.sr.ht/~onemoresuza/aba/commit/4bf9476ec50a272ee012eb6357b5d00169c8c9d2))

### Miscellaneous Tasks

- Fix typo on README ([8ec58c3](https://git.sr.ht/~onemoresuza/aba/commit/8ec58c39c070dc6cd83ae858976a73a95cd09b04))
- Fix typo on README ([9b10075](https://git.sr.ht/~onemoresuza/aba/commit/9b10075c83bae9bf3b1d64c0c93dc620243e875d))

## [0.3.0](https://git.sr.ht/~onemoresuza/aba/refs/0.3.0) - 2023-10-22

### Documentation

- Explain new `del` cmd behavior on manpage 1 ([07a93fa](https://git.sr.ht/~onemoresuza/aba/commit/07a93fadd7325c121b012440d3d47437cf8c0efb))

### Refactor

- [**breaking**] Turn `del` cmd into exact match ([97ab873](https://git.sr.ht/~onemoresuza/aba/commit/97ab873ab2a79e069767e917f750a0d03eeccf94))

## [0.2.0](https://git.sr.ht/~onemoresuza/aba/refs/0.2.0) - 2023-10-22

### Build

- Add cargo-edit to devShell ([a27e354](https://git.sr.ht/~onemoresuza/aba/commit/a27e3546d29544cd9fed46b211c4e88dbfd4c961))

### Miscellaneous Tasks

- Add TODO for not so needed mutability ([fd4ca8b](https://git.sr.ht/~onemoresuza/aba/commit/fd4ca8bafc4fca51ec056cd323d9d65db7246382))
- Add missing license identifier ([54b30d3](https://git.sr.ht/~onemoresuza/aba/commit/54b30d3715449106572364acf80e607c8f86cf3f))

### Refactor

- [**breaking**] Implement custom de- and serialzation ([73bc336](https://git.sr.ht/~onemoresuza/aba/commit/73bc336638b12975391166d40f1ccd051d26c4df))

## [0.1.1](https://git.sr.ht/~onemoresuza/aba/refs/0.1.1) - 2023-10-16

### Documentation

- Remove some long opts from `parse` cmd on manpage 1 ([8f6a453](https://git.sr.ht/~onemoresuza/aba/commit/8f6a453473f6a23c662dccc9ed2ce26a059d1315))

### Miscellaneous Tasks

- Add spdx identifier for CHANGELOG.md ([962f693](https://git.sr.ht/~onemoresuza/aba/commit/962f693a73d5de95f5abb19cff9c3c69a2f2b843))

### Refactor

- Remove some long opts from `parse` cmd ([42518d9](https://git.sr.ht/~onemoresuza/aba/commit/42518d96371c592811e2b3a262daf9bf9071aea8))

## [0.1.0](https://git.sr.ht/~onemoresuza/aba/refs/0.1.0) - 2023-10-16

### Bug Fixes

- Set correct description for `-f` option of `add` cmd ([01b057e](https://git.sr.ht/~onemoresuza/aba/commit/01b057e42786596a89dd5aed4ec7498cc968c52b))
- Erase file when `del` regex's matches every entry ([786e41d](https://git.sr.ht/~onemoresuza/aba/commit/786e41dc04c2ff7551d9a99851b9c1513f90946a))

### Build

- Add `install` recipe ([9607893](https://git.sr.ht/~onemoresuza/aba/commit/9607893d906d7891e280b2d5a2c9285eb196d9cf))
- Enable cargoAudit ([1dad680](https://git.sr.ht/~onemoresuza/aba/commit/1dad680b6025d8cd938f0d06c47405f438ac4468))
- Add manifest for nixos ([5a55576](https://git.sr.ht/~onemoresuza/aba/commit/5a55576c7512a1fba0a92a0b8ac074fbd12b5970))
- Fix typo on `reuse-lint` ([ec9480a](https://git.sr.ht/~onemoresuza/aba/commit/ec9480a0aeb0653a2b1fc956181bae3d88855d2b))
- Add cachix caching ([4a38b02](https://git.sr.ht/~onemoresuza/aba/commit/4a38b02cf472b69e40f3691a2c0cd5a37fe6bf31))
- Step into project dir before cachix setup ([745354f](https://git.sr.ht/~onemoresuza/aba/commit/745354f707741029a21812d0b5833667abdd5a0c))
- Add `cargo` `coreutils` to `genGitTagAndChangelog` app ([32a39ba](https://git.sr.ht/~onemoresuza/aba/commit/32a39ba910bb58997d2c21ef4340f2e64fa455d1))

### Documentation

- Create manpages ([c18d357](https://git.sr.ht/~onemoresuza/aba/commit/c18d357f87e9ee5ada48d2233b526f643cb23871))
- Add `parse` cmd to manpage 1 ([eb786d7](https://git.sr.ht/~onemoresuza/aba/commit/eb786d7aea5003ea64ea1427fcf16b4cba79afc2))
- Explain new file format on manpage 5 ([360c6b3](https://git.sr.ht/~onemoresuza/aba/commit/360c6b37df8de50d5aefe5ecc5d78d5bccd7faac))
- Update `parse` cmd options in manpage 1 ([becea84](https://git.sr.ht/~onemoresuza/aba/commit/becea84d54e5b636f1b9f6b44bf407fdcabca553))

### Features

- Add clap parser ([880ce99](https://git.sr.ht/~onemoresuza/aba/commit/880ce99c9fc4a27f19f90ec585f8dafceae69e28))
- Partially implement `AddressBook` ([134e55d](https://git.sr.ht/~onemoresuza/aba/commit/134e55d7c437f1fbdfab3b648b0af247de2df5af))
- Add rudimentary `add` cmd ([c041e7a](https://git.sr.ht/~onemoresuza/aba/commit/c041e7a67aac2d89cb4c204f928bacfbb508af09))
- Add `list` cmd ([0f8ab35](https://git.sr.ht/~onemoresuza/aba/commit/0f8ab35175307a6f81dd53f36507ea2be44093e6))
- Implement `parse` cmd partially ([7352c02](https://git.sr.ht/~onemoresuza/aba/commit/7352c02c32f43de1b88cff15ff7ceb7035008299))
- Implement `parse` cmd ([2330c12](https://git.sr.ht/~onemoresuza/aba/commit/2330c1216bbe2a4c01ce611bea7aac5fc951af63))
- Implement parsing of other headings ([4069dd8](https://git.sr.ht/~onemoresuza/aba/commit/4069dd89bfcf324b5905cc7c4388179c2cc07ddc))

### Miscellaneous Tasks

- Add `del` cmd ([d2a1cde](https://git.sr.ht/~onemoresuza/aba/commit/d2a1cdef077b41be0d664b473c7c57dd0c4b6e3a))
- Add README.md ([80f8cf5](https://git.sr.ht/~onemoresuza/aba/commit/80f8cf5339a475afda3437dd1a1841eafd036416))
- Remove generated manpages from repo ([76b43ce](https://git.sr.ht/~onemoresuza/aba/commit/76b43ce834e3cad7d90310b085ded3dde6634bea))
- Remove generated manpages from repo ([6dcaf89](https://git.sr.ht/~onemoresuza/aba/commit/6dcaf89ceb7cf45fad3ce1d57108fb11c0691855))
- Comply with reuse ([45c5480](https://git.sr.ht/~onemoresuza/aba/commit/45c5480f994f0202c5e93f7a5b92ae6f94b69291))
- Add git-cliff files ([966c6b0](https://git.sr.ht/~onemoresuza/aba/commit/966c6b044de58da8094532c9206ee68d50c89de4))

### Refactor

- Handle with non existing files ([329dc99](https://git.sr.ht/~onemoresuza/aba/commit/329dc997d734a53fdecd0623f52c96f05265df50))
- Split file creation in different method ([2aba7e5](https://git.sr.ht/~onemoresuza/aba/commit/2aba7e544a49c26e1b2aebae7d9def5a70facad8))
- Drop usage of `fs::OpenOptions` ([bfb96a0](https://git.sr.ht/~onemoresuza/aba/commit/bfb96a0938db202a007dfd09287d86e0221e3d98))
- Change address book file format ([cb1f2a7](https://git.sr.ht/~onemoresuza/aba/commit/cb1f2a7336bfaa5c544af0ca99df99802f6e16b4))
- Use regex to match `from` addresses ([#2](https://todo.sr.ht/~onemoresuza/aba/2)) ([a836a32](https://git.sr.ht/~onemoresuza/aba/commit/a836a32c72b1df133cfc7552073d321cffe64679))

### Testing

- Add unit tests for `AddressBook` assoc fns and methods ([c02452d](https://git.sr.ht/~onemoresuza/aba/commit/c02452d1aabad5aa1fd2cdf5ac9c3283558f3cdc))
- Add test for `del` cmd ([e35cdd4](https://git.sr.ht/~onemoresuza/aba/commit/e35cdd4b5258ec33c01877f6348d9c8e416cc790))

<!-- generated by git-cliff -->
