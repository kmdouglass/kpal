# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/kmdouglass/kpal/releases/tag/0.1.0

