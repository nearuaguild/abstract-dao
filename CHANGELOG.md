# Changelog

## [0.5.0](https://github.com/nearuaguild/abstract-dao/compare/v0.4.0...v0.5.0) (2024-10-07)


### ⚠ BREAKING CHANGES

* remove `Actor` enum completely & use a single `AccountId` instead of `Vec<Actor>`

### Miscellaneous Chores

* release 0.5.0 ([e3eb3d8](https://github.com/nearuaguild/abstract-dao/commit/e3eb3d8b4b468cc69dbe1250ee4d30f7028790f7))


### Documentation

* fill Readme file ([2d9b119](https://github.com/nearuaguild/abstract-dao/commit/2d9b1191c277b74ca57d05badaf74889cc3a6f56))
* leave comments to describe internal structures purpose ([a09875f](https://github.com/nearuaguild/abstract-dao/commit/a09875f355163d7401d35f82a3a9face8fb3a1da))


### Bug Fixes

* use `near_workspaces::compile_project()` to avoid errors due to the missing contract WASM file ([2e8b9a9](https://github.com/nearuaguild/abstract-dao/commit/2e8b9a9362ceb0617a7e371f16f6b909b6608cab))


### Code Refactoring

* remove `Actor` enum completely & use a single `AccountId` instead of `Vec&lt;Actor&gt;` ([ff85dca](https://github.com/nearuaguild/abstract-dao/commit/ff85dcaebbc0d95f700bda4a62a2b08bd3cc7eff))

## [0.4.0](https://github.com/nearuaguild/abstract-dao/compare/v0.3.0...v0.4.0) (2024-09-27)


### ⚠ BREAKING CHANGES

* rename field that counts request identifiers
* rename field that stores MPC contract id

### Miscellaneous Chores

* release 0.4.0 ([fc982f2](https://github.com/nearuaguild/abstract-dao/commit/fc982f2b38874ee7d4a6f5563c9b77f0862e7861))
* remove ability for owner to set another MPC contract ID ([ee21945](https://github.com/nearuaguild/abstract-dao/commit/ee219457ddf5e6e1d0788b4e4306cd310c969d64))


### Bug Fixes

* rename field that counts request identifiers ([ab33d9c](https://github.com/nearuaguild/abstract-dao/commit/ab33d9c2ba609cd067c3fc412865d49d1cb428ce))
* rename field that stores MPC contract id ([b44eba2](https://github.com/nearuaguild/abstract-dao/commit/b44eba217f6525a7d70ab483206f96e92afd9706))

## 0.3.0 (2024-09-18)


### Miscellaneous Chores

* add repositroy link ([2c796ea](https://github.com/nearuaguild/abstract-dao/commit/2c796eaf4ec562b23fd8c38c05fb92d53044987d))
* bump near-sdk to 5.3.0 ([7e02d68](https://github.com/nearuaguild/abstract-dao/commit/7e02d68e312843d027ec58c77a1c7c0edeb4796e))
* cargo description ([7f44a98](https://github.com/nearuaguild/abstract-dao/commit/7f44a980dc6b6e962572e0744ea023797b9061b7))
* leave TODOs for upcoming updates ([d20ee68](https://github.com/nearuaguild/abstract-dao/commit/d20ee68df304962679ac2d52c6a7c132ab2edd02))
* manual release version to 0.2.0 ([e9f71a6](https://github.com/nearuaguild/abstract-dao/commit/e9f71a688d3d14588624c58f177627882ec7f0d4))
* release 0.3.0 ([2b1366a](https://github.com/nearuaguild/abstract-dao/commit/2b1366a056b1c2b7b8314e2a559993d63e4605bc))
* remove default integration tests ([34615ec](https://github.com/nearuaguild/abstract-dao/commit/34615ecfc4619a9dc11730c70b4dd826f7d411d7))
* remove unnecessary packages & add `near-sdk` for unit-testing ([8b0e708](https://github.com/nearuaguild/abstract-dao/commit/8b0e708f6821da30aa8d1c92889b2d55b37ff0dc))
* update .gitignore ([d0af1d6](https://github.com/nearuaguild/abstract-dao/commit/d0af1d6a7cf22df4929aea408016aa15343a6ad7))
* update package name ([f944e37](https://github.com/nearuaguild/abstract-dao/commit/f944e37674e314e4494c23dd5d858963285899a7))


### Features

* construct data from Ethereum contract ABI ([7fc262e](https://github.com/nearuaguild/abstract-dao/commit/7fc262e71eb56d6cff6b98a787ffd827f59a6ece))
* implement core functionality of Abstract DAO ([db795f7](https://github.com/nearuaguild/abstract-dao/commit/db795f7c1cf24fb93bd5e3657a5ece2a08ff3840))
* introduce `get_signature` method that accepts ethereum transaction args ([8de851d](https://github.com/nearuaguild/abstract-dao/commit/8de851d20c2d6f657fc926d418e3ca8d9ae45d0a))


### Bug Fixes

* add ability to specify key_version in add_request input ([371fc47](https://github.com/nearuaguild/abstract-dao/commit/371fc47bc359e18dc584b08eb545dbd5ca2d7513))
* enable `abi` feature ([6eaf41a](https://github.com/nearuaguild/abstract-dao/commit/6eaf41a7d801fc4ba4c92731c3e9e75e743662e0))
* enable ABI support ([8ac5e85](https://github.com/nearuaguild/abstract-dao/commit/8ac5e85acde0e8e146d886b1b6bb466c84786934))
* impl JsonSchema trait for ABI ([65e7c27](https://github.com/nearuaguild/abstract-dao/commit/65e7c27c3b8130fd026fb64bce6a99c1f5025d99))
* reduce the amount of allowed actors ([9112f58](https://github.com/nearuaguild/abstract-dao/commit/9112f580ca36b973489678afb84bde24864227dc))
