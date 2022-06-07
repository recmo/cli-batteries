# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- Section names: Added, Changed, Deprecated, Removed, Fixed, Security -->

## [Unreleased]

### Fixed

* Bug where log-format `json` would not encode fields.

## [0.1.4] — 2022-05-30

### Added

* Tracing `ErrorLayer`

## [0.1.3] - 2022-05-26

### Fixed

* Bug where `await_shutdown` would not reset after `reset_shutdown`.

## [0.1.2] - 2022-05-25

### Added

* `mock_shutdown` function for testing.
* `mimalloc` and `metered_allocator` features.
* `default_from_structopt` macro.
* Implement basic traits on options.

## [0.1.1] — 2022-05-22

### Changed

* Exposed missing shutdown handling functions.
* Fixed incorrect issue url.
* Generalized app result to `Into<Report>`.

## [0.1.0] — 2022-05-21

Collected various common pieces of code for the v0.1.0 release.

<!-- links to version -->

[unreleased]: https://github.com/recmo/cli-batteries/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.2
[0.1.3]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.2
[0.1.2]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.2
[0.1.1]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.1
[0.1.0]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.0
