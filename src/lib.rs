//! # cognomen
//!
//! *Cognomen* — Latin for "an extra name given to a person or thing" — is a
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
//! #[derive(Cognomen)]
//! #[cognomen(snake_case)]
//! enum Mode {
//!     SingleProcess, // "single_process"
//!     MultiProcess,  // "multi_process"
//! }
//!
//! assert_eq!(Mode::SingleProcess.as_str(), "single_process");
//! assert_eq!(Mode::MultiProcess.label(), "multi_process");
//! ```
//!
//! Supported case styles:
//!
//! | `#[cognomen(...)]` | `VariantName` becomes |
//! |--------------------|-----------------------|
//! | `snake_case`       | `variant_name`        |
//! | `kebab-case`       | `variant-name`        |
//! | `camelCase`        | `variantName`         |
//! | `PascalCase`       | `VariantName`         |
//! | `SCREAMING_SNAKE_CASE` | `VARIANT_NAME`   |
//! | `lower`            | `variantname`         |
//! | `upper`            | `VARIANTNAME`         |
//!
//! # Requirements
//!
//! - Derive on **enums only**.
//! - **Unit variants only** (no fields).
//! - At least one variant.
//! - A `#[cognomen(<case style>)]` container attribute (also accepts the
//!   `#[cognomen(case = <style>)]` spelling).
//!
//! Violations are compile-time errors; the failure cases are pinned by
//! [trybuild](https://docs.rs/trybuild) UI tests under `tests/ui/`.

mod cognomen;

use proc_macro::TokenStream;

/// Derive `as_str` / `label` for unit-like enums with a case style.
///
/// Container attribute (required):
///
/// - `#[cognomen(snake_case)]`
/// - `#[cognomen(kebab-case)]`
/// - `#[cognomen(camelCase)]`
/// - `#[cognomen(PascalCase)]`
/// - `#[cognomen(SCREAMING_SNAKE_CASE)]`
/// - `#[cognomen(lower)]`
/// - `#[cognomen(upper)]`
///
/// The same styles are accepted with underscores (`kebab_case`) and in the
/// `#[cognomen(case = <style>)]` spelling.
///
/// Adds two `const fn`s that return the variant's stable label:
///
/// - [`label`](Self) — the primary accessor.
/// - `as_str` — an ergonomic alias suited to config and log call sites.
#[proc_macro_derive(Cognomen, attributes(cognomen))]
pub fn derive_cognomen(input: TokenStream) -> TokenStream {
    cognomen::derive(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
