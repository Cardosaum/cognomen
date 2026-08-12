//! Proc-macro implementation for [`cognomen`](https://docs.rs/cognomen).
//!
//! Depend on the `cognomen` crate, not this one. This crate exists so the
//! derive can live next to a `no_std` runtime (shared `FromLabelError`).

mod cognomen;

use proc_macro::TokenStream;

/// Derive stable, case-configured string labels for a unit-like enum.
///
/// # Container attribute
///
/// ```ignore
/// #[derive(Cognomen)]
/// #[cognomen(snake_case, kebab-case, prefix = "label", crate = ::cognomen)]
/// enum Mode { SingleProcess }
/// ```
///
/// - One or more case styles (required). The **first** is the default used by
///   `label()`, `as_str()`, `Display`, and serde serialization.
/// - `prefix = "..."`: stem for per-case methods (`label_snake`, ...). Must be
///   a non-empty ASCII identifier. Default: `label`.
/// - `crate = ::path`: crate path emitted in generated code. Default:
///   `::cognomen`. Set this when you re-export cognomen from another crate.
///
/// # Variant attribute
///
/// ```ignore
/// #[cognomen(rename = "io_error")]
/// IoFailed,
/// ```
///
/// Overrides the default label with that exact string and accepts it when
/// parsing. `label_snake()` and friends still convert from the ident.
///
/// # Generated items
///
/// - `const fn label(&self) -> &'static str`
/// - `const fn as_str(&self) -> &'static str`
/// - `const fn {prefix}_{case}(&self) -> &'static str` for each declared case
/// - `VARIANTS` / `LABELS` (non-generic enums)
/// - `Display`, `AsRef<str>`, `PartialEq<str>`
/// - `TryFrom<&str>`, `FromStr`, `from_label` (feature `alloc`)
/// - `Serialize` / `Deserialize` (feature `serde`)
///
/// See the [`cognomen`](https://docs.rs/cognomen) crate docs for case styles,
/// features, and `no_std`.
#[proc_macro_derive(Cognomen, attributes(cognomen))]
pub fn derive_cognomen(input: TokenStream) -> TokenStream {
    cognomen::derive(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
