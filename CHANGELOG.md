# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2019-11-23
### Added
- Added a `CHANGELOG.md` file to track project changes.
- An integration test that checks all the endpoints in the user API is now running on the CI.

### Removed
- All dependencies on an external database. This should increase portability and simplify some
  aspects of the code because `kpald` now relies on native Rust data structures.

[Unreleased]: https://github.com/kmdouglass/kpal

