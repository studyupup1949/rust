# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/Unleash/actix-allow-deny-middleware/compare/v0.1.1...v0.1.2) - 2025-07-01

### 💼 Other
- Add test to confirm ports > 32768 works ([#8](https://github.com/unleash/actix-allow-deny-middleware/issues/8)) (by @chriswk) - #8

### Dependency updates
- bump actix-service from 2.0.2 to 2.0.3 ([#2](https://github.com/unleash/actix-allow-deny-middleware/issues/2)) (by @dependabot[bot]) - #2
- bump actions/create-github-app-token from 1 to 2 ([#5](https://github.com/unleash/actix-allow-deny-middleware/issues/5)) (by @dependabot[bot]) - #5
- bump actix-web from 4.9.0 to 4.11.0 ([#7](https://github.com/unleash/actix-allow-deny-middleware/issues/7)) (by @dependabot[bot]) - #7

### Contributors

* @dependabot[bot]
* @chriswk

## [0.1.1](https://github.com/Unleash/actix-allow-deny-middleware/compare/v0.1.0...v0.1.1) - 2025-03-03

### 🐛 Bug Fixes
- deny middleware only denies if it has ip and entries in deny list (by @chriswk)

### Contributors

* @chriswk

## [0.1.0](https://github.com/Unleash/actix-allow-deny-middleware/releases/tag/v0.1.0) - 2025-02-28

### 🚀 Features
- added allow and deny middlewares (by @chriswk)
- Make it easy to build Allow and Disallow middlewares from single ips or lists of ips (by @chriswk)
- added allow and disallow list middlewares (by @chriswk)

### 💼 Other
- Setup CI + release-plz (by @chriswk)
- initial commit (by @chriswk)

### ⚙️ Miscellaneous Tasks
- *(ci)* setup semver check for releases (by @chriswk)
- *(release)* Added changelog file (by @chriswk)
- transfer license to Bricks Software (by @chriswk)
- *(ci)* update release-plz with new crate name (by @chriswk)
- added tests for allow middleware (by @chriswk)
- *(ci)* Setup release-plz (by @chriswk)
- *(clippy)* thanks (by @chriswk)
- added dependabot and mergify setup (by @chriswk)

### Contributors

* @chriswk

## v0.2.0 (2026-06-05)

### Chore

 - <csr-id-8ce7992a086c3eff92f4f679bba5f73cc9579686/> update dependencies

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 339 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update dependencies ([`8ce7992`](https://github.com/chriswk/actix-allow-deny-middleware/commit/8ce7992a086c3eff92f4f679bba5f73cc9579686))
</details>

## v0.1.2 (2025-07-01)

### Chore

 - <csr-id-3a78b5ab4001366636d1d06009561818631ad20c/> release v0.1.2

### Other

 - <csr-id-d9c6f654e9dee585629da932662e3b641b7c3163/> bump actix-service from 2.0.2 to 2.0.3
   Bumps [actix-service](https://github.com/actix/actix-net) from 2.0.2 to 2.0.3.
   - [Release notes](https://github.com/actix/actix-net/releases)
   - [Commits](https://github.com/actix/actix-net/compare/rt-v2.0.2...service-v2.0.3)
   
   ---
   updated-dependencies:
   - dependency-name: actix-service
     dependency-type: direct:production
     update-type: version-update:semver-patch
   ...
 - <csr-id-3e8e198c3758a2e59b26c9274ac23db7d8443564/> Add test to confirm ports > 32768 works
 - <csr-id-3e182e3585291b5eb4193e407711a35521ef64f7/> bump actions/create-github-app-token from 1 to 2
   Bumps [actions/create-github-app-token](https://github.com/actions/create-github-app-token) from 1 to 2.
   - [Release notes](https://github.com/actions/create-github-app-token/releases)
   - [Commits](https://github.com/actions/create-github-app-token/compare/v1...v2)
   
   ---
   updated-dependencies:
   - dependency-name: actions/create-github-app-token
     dependency-version: '2'
     dependency-type: direct:production
     update-type: version-update:semver-major
   ...
 - <csr-id-2d07994fbcf086ec76c1bf53d9adf95f063aadae/> bump actix-web from 4.9.0 to 4.11.0
   Bumps [actix-web](https://github.com/actix/actix-web) from 4.9.0 to 4.11.0.
   - [Release notes](https://github.com/actix/actix-web/releases)
   - [Changelog](https://github.com/actix/actix-web/blob/master/CHANGES.md)
   - [Commits](https://github.com/actix/actix-web/compare/web-v4.9.0...web-v4.11.0)
   
   ---
   updated-dependencies:
   - dependency-name: actix-web
     dependency-version: 4.11.0
     dependency-type: direct:production
     update-type: version-update:semver-minor
   ...

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 5 unique issues were worked on: [#2](https://github.com/chriswk/actix-allow-deny-middleware/issues/2), [#5](https://github.com/chriswk/actix-allow-deny-middleware/issues/5), [#7](https://github.com/chriswk/actix-allow-deny-middleware/issues/7), [#8](https://github.com/chriswk/actix-allow-deny-middleware/issues/8), [#9](https://github.com/chriswk/actix-allow-deny-middleware/issues/9)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#2](https://github.com/chriswk/actix-allow-deny-middleware/issues/2)**
    - Bump actix-service from 2.0.2 to 2.0.3 ([`d9c6f65`](https://github.com/chriswk/actix-allow-deny-middleware/commit/d9c6f654e9dee585629da932662e3b641b7c3163))
 * **[#5](https://github.com/chriswk/actix-allow-deny-middleware/issues/5)**
    - Bump actions/create-github-app-token from 1 to 2 ([`3e182e3`](https://github.com/chriswk/actix-allow-deny-middleware/commit/3e182e3585291b5eb4193e407711a35521ef64f7))
 * **[#7](https://github.com/chriswk/actix-allow-deny-middleware/issues/7)**
    - Bump actix-web from 4.9.0 to 4.11.0 ([`2d07994`](https://github.com/chriswk/actix-allow-deny-middleware/commit/2d07994fbcf086ec76c1bf53d9adf95f063aadae))
 * **[#8](https://github.com/chriswk/actix-allow-deny-middleware/issues/8)**
    - Add test to confirm ports > 32768 works ([`3e8e198`](https://github.com/chriswk/actix-allow-deny-middleware/commit/3e8e198c3758a2e59b26c9274ac23db7d8443564))
 * **[#9](https://github.com/chriswk/actix-allow-deny-middleware/issues/9)**
    - Release v0.1.2 ([`3a78b5a`](https://github.com/chriswk/actix-allow-deny-middleware/commit/3a78b5ab4001366636d1d06009561818631ad20c))
 * **Uncategorized**
    - Merge pull request #1 from Unleash/release-plz-2025-02-28T14-13-31Z ([`4ccb137`](https://github.com/chriswk/actix-allow-deny-middleware/commit/4ccb1376f967e40e729ae6c91e16fd41e2c46c3d))
</details>

## v0.1.1 (2025-03-03)

### Chore

 - <csr-id-29bfd9d1cf193e6a40550aa129b0b9eae57ac9cd/> release v0.1.1
 - <csr-id-40dfccf2bfba4612cfc84d7950bbeeb6a64d4c5e/> Removed unnecessary dependency on futures

### Bug Fixes

 - <csr-id-0ae4883f6a8bb4cbf9ed7b3166aaf30c75f607e9/> deny middleware only denies if it has ip and entries in deny list

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release v0.1.1 ([`29bfd9d`](https://github.com/chriswk/actix-allow-deny-middleware/commit/29bfd9d1cf193e6a40550aa129b0b9eae57ac9cd))
    - Deny middleware only denies if it has ip and entries in deny list ([`0ae4883`](https://github.com/chriswk/actix-allow-deny-middleware/commit/0ae4883f6a8bb4cbf9ed7b3166aaf30c75f607e9))
    - Removed unnecessary dependency on futures ([`40dfccf`](https://github.com/chriswk/actix-allow-deny-middleware/commit/40dfccf2bfba4612cfc84d7950bbeeb6a64d4c5e))
</details>

## v0.1.0 (2025-02-28)

### Chore

 - <csr-id-5d5ee0c6837fb4bdbe5f8249f9b7a4f5a2aa8298/> Added description to package
 - <csr-id-be1d60e2dd1b9769c2ba47af3fab75cf20fb4af0/> Updated examples for deny listing
 - <csr-id-feb62073dc59fcc653a7c219ab75b43991ca4d5d/> Make sure release-plz can read and update CHANGELOG.md
 - <csr-id-76d19177181a15261a43d6e126cbc290edf928ff/> setup semver check for releases
 - <csr-id-737662bc413639ea77cd7d1ebd3c2a8c539a89a9/> Added changelog file
 - <csr-id-1a7686ff82c8da06048f8483a0f396bf38d58790/> transfer license to Bricks Software
 - <csr-id-91f944108920b658ad8c9567f7f85692fb92e2a3/> update release-plz with new crate name
 - <csr-id-c2c0ab384bc84428abb7ac419271a9b03ae7cd0b/> added tests for allow middleware
 - <csr-id-d1538fef1b48e6a05cd98265dc33da1946448ba6/> Setup release-plz
 - <csr-id-ada277d38ef44aad868d61fb6f91a6ca7955228d/> thanks
 - <csr-id-0125db217b7d42b9bd7e6b1a688e82bed2172f2c/> added dependabot and mergify setup

### New Features

 - <csr-id-1a1225b4b7a95f9d4605adbcd9c413f6e06041b8/> added allow and deny middlewares
 - <csr-id-df3001e308758d6af31157f24bb4ae07890c3c06/> Make it easy to build Allow and Disallow middlewares from single ips or lists of ips
 - <csr-id-b87c7bb73a46d88fdb31d51821542d3496a3a2bc/> added allow and disallow list middlewares

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release.
 - 14 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Added description to package ([`5d5ee0c`](https://github.com/chriswk/actix-allow-deny-middleware/commit/5d5ee0c6837fb4bdbe5f8249f9b7a4f5a2aa8298))
    - Updated examples for deny listing ([`be1d60e`](https://github.com/chriswk/actix-allow-deny-middleware/commit/be1d60e2dd1b9769c2ba47af3fab75cf20fb4af0))
    - Make sure release-plz can read and update CHANGELOG.md ([`feb6207`](https://github.com/chriswk/actix-allow-deny-middleware/commit/feb62073dc59fcc653a7c219ab75b43991ca4d5d))
    - Setup semver check for releases ([`76d1917`](https://github.com/chriswk/actix-allow-deny-middleware/commit/76d19177181a15261a43d6e126cbc290edf928ff))
    - Added changelog file ([`737662b`](https://github.com/chriswk/actix-allow-deny-middleware/commit/737662bc413639ea77cd7d1ebd3c2a8c539a89a9))
    - Transfer license to Bricks Software ([`1a7686f`](https://github.com/chriswk/actix-allow-deny-middleware/commit/1a7686ff82c8da06048f8483a0f396bf38d58790))
    - Update release-plz with new crate name ([`91f9441`](https://github.com/chriswk/actix-allow-deny-middleware/commit/91f944108920b658ad8c9567f7f85692fb92e2a3))
    - Added allow and deny middlewares ([`1a1225b`](https://github.com/chriswk/actix-allow-deny-middleware/commit/1a1225b4b7a95f9d4605adbcd9c413f6e06041b8))
    - Added tests for allow middleware ([`c2c0ab3`](https://github.com/chriswk/actix-allow-deny-middleware/commit/c2c0ab384bc84428abb7ac419271a9b03ae7cd0b))
    - Make it easy to build Allow and Disallow middlewares from single ips or lists of ips ([`df3001e`](https://github.com/chriswk/actix-allow-deny-middleware/commit/df3001e308758d6af31157f24bb4ae07890c3c06))
    - Setup release-plz ([`d1538fe`](https://github.com/chriswk/actix-allow-deny-middleware/commit/d1538fef1b48e6a05cd98265dc33da1946448ba6))
    - Thanks ([`ada277d`](https://github.com/chriswk/actix-allow-deny-middleware/commit/ada277d38ef44aad868d61fb6f91a6ca7955228d))
    - Added allow and disallow list middlewares ([`b87c7bb`](https://github.com/chriswk/actix-allow-deny-middleware/commit/b87c7bb73a46d88fdb31d51821542d3496a3a2bc))
    - Added dependabot and mergify setup ([`0125db2`](https://github.com/chriswk/actix-allow-deny-middleware/commit/0125db217b7d42b9bd7e6b1a688e82bed2172f2c))
    - Setup CI + release-plz ([`3da31d0`](https://github.com/chriswk/actix-allow-deny-middleware/commit/3da31d03e110488d814fe970cb005e69a9962b3a))
    - Initial commit ([`efa0511`](https://github.com/chriswk/actix-allow-deny-middleware/commit/efa051155df0446983950a7de3d9f2ce5cb1afe4))
</details>

