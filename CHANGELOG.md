# Changelog

Notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0]

### Added

* Links to `pyproject.toml`
* Ability to supply additional rules to default rule set
* Optional parameter `prefilter` to exit early if no concepts are identified in a text
* Github Pages documentation + links in pyproject.toml

### Changed

* Handle special case where search term (i.e., concept) is *also* a negation term (e.g., excluding lists of drugs as
  hypothetical)

## [0.1.0] - 2026-09-11

### Added

- Base implementation for negex/context for concept detection in text
- Useful data wrangling extension expressions to `polars` for `starts_with_any` and `ends_with_any`
- Github setup + pypi releases

[unreleased]: https://github.com/kpwhri/polars_cnlp/compare/v0.2.0...HEAD

[0.2.0]: https://github.com/kpwhri/polars_cnlp/compare/v0.1.0...v0.2.0

[0.1.0]: https://github.com/kpwhri/polars_cnlp/releases/tag/v0.1.0