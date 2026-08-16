# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/Cardosaum/cognomen/compare/cognomen-macros-v0.2.0...cognomen-macros-v0.2.1) - 2026-08-16

### Other

- Put tables on Variants and stop implementing Display

### Changed

- emit `VARIANTS` / `LABELS` on `cognomen::Variants`, not as inherent items
- do not implement `Display`

### Fixed

- `TryFrom` / `FromStr` now name `FromLabelError` instead of `Self::Error` / `Self::Err`, so variants named `Error` or `Err` compile

## [0.2.0](https://github.com/Cardosaum/cognomen/releases/tag/cognomen-macros-v0.2.0) - 2026-08-16

### Added

- extra string methods from `#[cognomen(name = "...")]`; omitted variants use `as_str()` unless the enum sets a default

## [0.1.0](https://github.com/Cardosaum/cognomen/releases/tag/cognomen-macros-v0.1.0) - 2026-08-15

### Added

- split no_std runtime and expand building-block surface

### Fixed

- *(cognomen)* emit parse with core on no_std

### Other

- *(release)* add release-plz and crates.io package metadata
- *(lint)* move missing_docs into workspace Cargo lints
- *(cognomen)* prune derive duplication
