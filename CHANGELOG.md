# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2020-03-29
### Added
- A new error code was added to the `kpal-plugin` crate called `STRING_CONVERSION_ERR`.
- A new module called `integrations` was added. Its purpose is to make it easier to create
  different types of user APIs and to decouple the KPAL models from their input/output
  representations.
- A crate-level docstring was added to the `kpal` crate.

### Changed
- The integration tests were refactored and expanded to cover more use cases.
- JSON, not text, is now returned in HTTP error responses.
- The internal handling of matches on pairs of enums was redone. This has no effect on users, but
  for developers the compiler should now be able to remind you when you forget to update methods
  involving matches on pairs after introducing a new enum variant for `Attribute`, `Value`, etc.
- Error handling in `kpal-plugin` and `kpal-gpio-cdev` was refactored to condense the number of
  error types exported by `kpal-plugin` to one.
  
### Fixed
- `kpal-gpio-cdev` now returns the correct `PLUGIN_INIT_ERR` error code when the plugin fails to
  initialize. Previously it was returning `NUMERIC_CONVERSION_ERR`.

## [0.2.2] - 2020-03-07
### Changed
- Module structures were rerranged so that module APIs are no longer nested; there is only layer of
  modules below the KPAL root layer of modules.
- All modules, including those hidden inside the public ones, now export only one Error type.

### Fixed
- `kpal` and `kpal-gpio-cdev` now depend on the correct version of `kpal-plugin`.

## [0.2.1] - 2020-03-01
### Changed
- The method to create new peripherals now accepts a JSON object and not an array for the pre-init
  attributes.
- The `links` field of a peripheral JSON resource is now renamed to `attributes`. Its name in the
  Rust code remains unchanged.

### Fixed
- A plugin executor can no longer get stuck in an infinite loop when initializing if a plugin
  provides no attributes. This was fixed by adding two new methods to the plugin API:
  `attribute_count` and `attribute_ids`.
- A delay was added between publishing jobs on the CI so that crates.io would have time to update
  before subsequent crates were published.

## [0.2.0] - 2020-02-09
### Added
- A Dockerfile was added at
  [resources/docker/rust-linux-x86_64/Dockerfile](resources/docker/rust-linux-x86_64/Dockerfile)
  that defines the image in which the x86_64 CI jobs are run.
- A new workflow was added to the CI configuration for releases. The new workflow is called `main`
  and has two pathways. One is triggered by a commit and one by pushing a tag to the Git repo. The
  pathway that is triggered by a commit contains a subset of the jobs contained by the one that is
  triggered by tag. This enables a release-from-tag strategy.
- A build cache was added to the CI that includes the `~/.cargo` and `target` folders.
- A job was added to the CI to build artifacts for the ARMv7 architecture (the one used by the
  Raspberry Pi). The Dockerfiles for the builder and tester are located
  [here](resources/docker/rust_cross_armv7-linux-x86_64/Dockerfile) and
  [here](resources/docker/kpal_tester-linux-armv7/Dockerfile), respectively.
- Added the Dependabot dependencey management service to the GitHub repo.
- Added steps to the CI to check that the code has been linted with `cargo clippy`.
- A new set of jobs was added to the CI to build and publish Dockerfiles to Docker Hub.
- `kpal-plugin` now exposes a `KpalLibraryInit` type that is used by the daemon to initialize
  plugin libraries. Previously, the function signature was hard-coded.
- A new macro called `declare_plugin` was added to the `kpal-plugin` library. This macro takes care
  of initializing the FFI code for plugins so that plugin authors do not have to.
- Callbacks were added to the `kpal-plugin` library, making it easier to isolate the parts of the
  plugin code that communicate with the hardware.
- Peripherals may how have string attributes that contain any character within the ASCII character
  set except for the null byte.
- Peripherals now have `pre-init` attributes. These allow you to set attribute values before the
  plugin is initialized, which improves the ergonomics of writing plugins.
- A new unsigned integer Attribute type was added.

### Changed
- All artifacts are now built on the CI with the `--release` profile.
- The entire KPAL codebase is now linted with `clippy`.
- The `plugins::driver` and `plugins::init` modules were moved into methods of the Executor struct
  that is provided by the `plugins::executor` module.
- All errors in the `plugins` module were consolidated and moved into a `plugins::errors`
  submodule. Likewise, all errors in the `web` module were consolidated and moved into a
  `web::errors` submodule.
- The `Peripheral` in the `kpal-plugin` crate was renamed to `PluginData` to avoid confusion with
  the `Peripheral` of the user API and to emphasize that it holds the state of a plugin.
- Many methods in the `PluginAPI` trait now have default implementations so that they are no longer
  required in a plugin library.
- Changed the name of the `kpal_plugin_init` FFI function to `kpal_plugin_new`.

### Fixed
- The integration and unit tests no longer look in only the `target/debug` directory for test
  artifacts. Instead, they search for artifacts in the parent folder and subfolders of the
  currently-running test binary.
- Fixed a unit test that was not compiling on 32-bit Linux platforms due to a difference in integer
  size as compared to 64-bit systems.
- The description of kpald in the --help text now matches the one on GitHub and in Cargo.toml.

## [0.1.0] - 2019-12-14
### Added
- A `CHANGELOG.md` file to track project changes.
- An integration test that checks all the endpoints in the user API is now running on the CI.
- The `PATCH /api/v0/peripherals/*/attributes/*` endpoint and the ability to set attribute values.
- A new crate called `kpal-gpio-cdev` was created that enables control of the output over a single
  GPIO pin.

### Changed
- `Scheduler` was renamed to `Executor`.
- `Attribute::from` was renamed to `Attribute::new` to avoid confusion with the `From` trait.
- `init::library` was renamed to `init::libraries` for consistency with `init::transmitters`.
- `TSLibrary` was moved to the `init::libraries` module for consistency with the location of
  `init::transmitters::Transmitters`.
- Error handling in the `kpal-plugins` crate is now greatly improved. Plugin authors can now write
  their own error types that implement both Rust's `Error` trait and the new `PluginError` trait
  provided by `kpal-plugin`.

### Removed
- All dependencies on an external database. This should increase portability and simplify some
  aspects of the code because `kpald` now relies on native Rust data structures.

### Fixed
- HTTP error codes are no longer all 404s.
- The Daemon now uses C datatypes instead of Rust datatypes for values. This fixes an issue where
  KPAL would not compile on 32-bit processors due to different integer sizes.

[Unreleased]: https://github.com/kmdouglass/kpal/compare/0.2.2...HEAD
[0.2.2]: https://github.com/kmdouglass/kpal/releases/tag/0.2.2
[0.2.1]: https://github.com/kmdouglass/kpal/releases/tag/0.2.1
[0.2.0]: https://github.com/kmdouglass/kpal/releases/tag/0.2.0
[0.1.0]: https://github.com/kmdouglass/kpal/releases/tag/0.1.0

