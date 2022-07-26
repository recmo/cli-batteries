# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- Section names: Added, Changed, Deprecated, Removed, Fixed, Security -->

## [Unreleased]

## [0.3.1] — 2022-07-26

### Added

* Field formatter for tiny
* Read verbosity from environment
* Allow specifying additional crates to consider local

### Changed

* Tiny is now the default log format.

## [0.3.0] — 2022-07-25

### Added

* Heartbeat log messages every 5 min.
* Log format `tiny` that is more condensed than tracing's `compact`.
* `--trace-flame` to store traces in a flamegraph output file.
* Capture `log` crate events.

### Changed

* Signal handling behind `signals` flag. Signals are no longer handled by default.
* Log output now goes to stderr, not stdout.
* Better formatting of span events.
* Crate events are treated as app local.

## [0.2.1] — 2022-07-06

### Fixed

* Apply log filter to spans and traces

## [0.2.0] — 2022-07-01

### Added

* Entity resource attributes for trace submission.

### Changed

* Removed datadog trace export support (which was broken).
* Deprecated stuctopt in favor of clap derive.

## [0.1.7] — 2022-06-24

### Added

* Timeouts on the `datadog` reqwest client.

### Fixed

* Workaround for `datadog` url issue.

## [0.1.6] — 2022-06-20

### Added

* Added `opentelemetry` layer with `otlp` or `datadog` backends.

## Changed

* The tracing log is now constructed inside the Tokio runtime. This is for OpenTelemetry which requires an active runtime during construction.
* Feature names are now kebab-case for consistency.

## [0.1.5] — 2022-06-07

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

[unreleased]: https://github.com/recmo/cli-batteries/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/recmo/cli-batteries/releases/tag/v0.3.1
[0.3.0]: https://github.com/recmo/cli-batteries/releases/tag/v0.3.0
[0.2.1]: https://github.com/recmo/cli-batteries/releases/tag/v0.2.1
[0.2.0]: https://github.com/recmo/cli-batteries/releases/tag/v0.2.0
[0.1.7]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.7
[0.1.6]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.6
[0.1.5]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.5
[0.1.4]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.4
[0.1.3]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.3
[0.1.2]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.2
[0.1.1]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.1
[0.1.0]: https://github.com/recmo/cli-batteries/releases/tag/v0.1.0
