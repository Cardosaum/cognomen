# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/Cardosaum/cognomen/compare/cognomen-v0.4.0...cognomen-v0.5.0) - 2026-08-16

### Other

- Interpolate extras as Formatted trait methods ([#10](https://github.com/Cardosaum/cognomen/pull/10))

### Added

- fielded variants: `Label` ignores the payload; `{field}` in a variant extra
  interpolates into `Formatted`
- tuple-index placeholders (`{0}`) and `{{` / `}}` escapes in extras
- `Label`, `FromLabel`, `Extra<K>`, and named extra traits (`Reason`,
  `Blurb`, `Hint`, `Help`) in this crate

### Changed

- parse, `Variants`, and serde-in are omitted when any variant has a payload
  (a label cannot reconstruct fields)
- extras always return `Formatted` (static text is one literal). Adding
  `{field}` does not change the signature
- labels and extras are trait items, not inherent methods on `E`. Import
  `Label` / `Reason` / ... or use UFCS. `prefix` no longer names methods;
  use `Label::in_case`
- `from_label` lives on `FromLabel` (fieldless only)

### Notes

- `numbered` still has inherent `number()`; a follow-up should move that
  onto a trait the same way

## [0.4.0](https://github.com/Cardosaum/cognomen/releases/tag/cognomen-v0.4.0) - 2026-08-16

### Added

- `clap` feature: `T::value_parser()` after `use cognomen::clap::ArgType`, so a `no_std` enum crate does not depend on clap

## [0.3.0](https://github.com/Cardosaum/cognomen/releases/tag/cognomen-v0.3.0) - 2026-08-16

### Changed

- `VARIANTS` and `LABELS` live on the `Variants` trait, not as inherent items, so they cannot clash with another derive or user code
- Cognomen no longer implements `Display`; print `e.label()` / `e.as_str()`

### Fixed

- `TryFrom` / `FromStr` now name `FromLabelError` instead of `Self::Error` / `Self::Err`, so variants named `Error` or `Err` compile
- `Deserialize` now keeps the enum's generics, so `enum Flag<const N: usize>` compiles with `serde`

## [0.2.0](https://github.com/Cardosaum/cognomen/releases/tag/cognomen-v0.2.0) - 2026-08-16

### Added

- extra string methods from `#[cognomen(name = "...")]`; omitted variants use `as_str()` unless the enum sets a default

## [0.1.0](https://github.com/Cardosaum/cognomen/releases/tag/cognomen-v0.1.0) - 2026-08-15

### Added

- split no_std runtime and expand building-block surface
- *(cognomen)* add customizable prefix for accessor methods
- *(cognomen)* derive the reverse string path back to the enum
- *(cognomen)* support multiple case styles per enum
- *(cognomen)* publish-ready Labeled derive crate

### Fixed

- *(cognomen)* emit parse with core on no_std

### Other

- *(release)* add release-plz and crates.io package metadata
- *(lint)* move missing_docs into workspace Cargo lints
- *(cognomen)* drop as_str, shrink derive surface
- *(lint)* scope ast-grep config under ast-grep/
- *(cognomen)* write prose plainly instead of using ASCII dash substitutes
- *(cognomen)* use Matheus Cardoso as copyright holder
- *(license)* use byte-exact official Apache-2.0 text
- *(lint)* enforce ASCII-only source and no em dashes in Markdown
- *(cognomen)* rename Labeled derive to Cognomen
- *(cognomen)* set MSRV to 1.71.1 (cargo-msrv)
- *(cognomen)* dual-license MIT OR Apache-2.0
- add test, clippy/fmt, and MSRV pipeline
