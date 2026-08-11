//! # cognomen
//!
//! *Cognomen* is Latin for "an extra name given to a person or thing". It is a
//! proc-macro derive that gives every unit-like variant of an enum a second
//! name: a stable, case-configured string label exposed via [`label`](Cognomen).
//!
//! This is the layer between an enum's Rust identifier and the strings a
//! config file, log line, or wire message actually carries. Case conversion is
//! done once at compile time and emitted as a `&'static str`, so there is no
//! runtime cost.
//!
//! ```rust
//! use cognomen::Cognomen;
//!
//! // The first case listed is the default; every case gets a `<prefix>_<case>`
//! // accessor. `label()` / `as_str()` alias the default.
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
//! #[cognomen(snake_case, kebab-case)]
//! enum Mode {
//!     SingleProcess, // "single_process" / "single-process"
//!     MultiProcess,  // "multi_process"  / "multi-process"
//! }
//!
//! assert_eq!(Mode::SingleProcess.label(), "single_process");
//! assert_eq!(Mode::MultiProcess.as_str(), "multi_process");
//! assert_eq!(Mode::SingleProcess.label_kebab(), "single-process");
//! assert_eq!(Mode::MultiProcess.label_kebab(), "multi-process");
//!
//! // Parse any declared case back to the variant.
//! assert_eq!(Mode::try_from("single_process"), Ok(Mode::SingleProcess));
//! assert_eq!(Mode::try_from("single-process"), Ok(Mode::SingleProcess));
//! assert_eq!("multi_process".parse::<Mode>(), Ok(Mode::MultiProcess));
//! assert!(Mode::try_from("hovercraft").is_err());
//! ```
//!
//! List more than one case comma-separated in the `#[cognomen(...)]` attribute;
//! the **first** is the default returned by `label()` / `as_str()`.
//!
//! The derive also implements the reverse path so the round trip is complete.
//! `TryFrom<&str>` and `FromStr` accept a string in any declared case and
//! return the variant, or a `FromLabelError` when nothing matches.
//!
//! Supported case styles and their accessors:
//!
//! | `#[cognomen(...)]`       | method                  | `VariantName` becomes |
//! |--------------------------|-------------------------|-----------------------|
//! | `snake_case`             | `label_snake`           | `variant_name`        |
//! | `kebab-case`             | `label_kebab`           | `variant-name`        |
//! | `camelCase`              | `label_camel`           | `variantName`         |
//! | `PascalCase`             | `label_pascal`          | `VariantName`         |
//! | `SCREAMING_SNAKE_CASE`   | `label_screaming_snake` | `VARIANT_NAME`        |
//! | `lower`                  | `label_lower`           | `variantname`         |
//! | `upper`                  | `label_upper`           | `VARIANTNAME`         |
//!
//! # Requirements
//!
//! - Derive on **enums only**.
//! - **Unit variants only** (no fields).
//! - At least one variant.
//! - At least one `#[cognomen(<case style>)]` container attribute with one or
//!   more comma-separated cases (e.g. `#[cognomen(snake_case, kebab-case)]`).
//!   The **first** case is the default.
//!
//! Violations are compile-time errors; the failure cases are pinned by
//! [trybuild](https://docs.rs/trybuild) UI tests under `tests/ui/`.

mod cognomen;

use proc_macro::TokenStream;

/// Derive `label` / `as_str` for unit-like enums, plus a `label_<case>`
/// accessor for every declared case.
///
/// Container attribute (required):
///
/// - `#[cognomen(snake_case)]`
/// - `#[cognomen(snake_case, kebab-case)]`: one or more comma-separated
///   cases; the **first** is the default.
///
/// Supported styles: `snake_case`, `kebab-case`, `camelCase`, `PascalCase`,
/// `SCREAMING_SNAKE_CASE`, `lower`, `upper` (also with underscores, e.g.
/// `kebab_case`).
///
/// Every case in the list generates a `<prefix>_<case>` const fn
/// (`label_snake`, `label_kebab`, `label_pascal`, ...). [`label`](Self) and
/// `as_str` are aliases for the default (first) case.
///
/// Optional `prefix = "..."` changes the accessor name prefix
/// (default `"label"`). Example:
/// `#[cognomen(snake_case, prefix = "my_label")]` generates
/// `my_label_snake()` instead of `label_snake()`.
#[proc_macro_derive(Cognomen, attributes(cognomen))]
pub fn derive_cognomen(input: TokenStream) -> TokenStream {
    cognomen::derive(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
