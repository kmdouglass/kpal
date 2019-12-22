# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2019-12-22
### Added
- A Dockerfile was added at
  [resources/docker/rust-linux-x86_64/Dockerfile](resources/docker/rust-linux-x86_64/Dockerfile)
  that defines the image in which the x86_64 CI jobs are run. The Docker file is automatically
  built and hosted on DockerHub.
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

### Changed
- All artifacts are now built on the CI with the `--release` profile.

### Fixed
- The integration and unit tests no longer look in only the `target/debug` directory for test
  artifacts. Instead, they search for artifacts in the parent folder and subfolders of the
  currently-running test binary.
- Fixed a unit test that was not compiling on 32-bit Linux platforms due to a difference in integer
  size as compared to 64-bit systems.

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

[Unreleased]: https://github.com/kmdouglass/kpal/compare/0.1.0...HEAD
[0.1.0]: https://github.com/kmdouglass/kpal/releases/tag/0.1.0

