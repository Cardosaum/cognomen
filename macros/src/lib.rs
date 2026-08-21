//! Proc-macro implementation for [`cognomen`](https://docs.rs/cognomen).
//!
//! Depend on the `cognomen` crate, not this one. This crate is the host
//! proc-macro; generated items use `core` so the runtime can be `no_std`.

mod cognomen;

use proc_macro::TokenStream;

/// Derive stable, case-configured string labels for an enum.
///
/// # Container attribute
///
/// ```ignore
/// #[derive(Cognomen)]
/// #[cognomen(snake_case, kebab-case, crate = ::cognomen)]
/// enum Mode { SingleProcess }
/// ```
///
/// - One or more case styles (required). The **first** is the default used by
///   [`cognomen::Label`], and serde serialization.
/// - `prefix = "..."`: accepted for compatibility; case accessors live on
///   [`cognomen::Label`].
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
/// parsing. [`cognomen::Label::in_case`] still converts from the ident.
///
/// `#[cognomen(alias = "main")]` adds a parse-in string only. It does not
/// change `label()`, `as_str()`, serde-out, `PartialEq<str>`, or `in_case`.
/// Repeat the key for more than one alias. Empty `""` is a compile error.
///
/// `#[cognomen(unknown)]` on exactly one unit variant of a fieldless enum
/// sends unmatched parse and serde-in to that variant. The unmatched string
/// is not stored. clap `value_parser` still rejects unmatched input.
///
/// # Extras
///
/// Any other `name = "..."` is an extra. Known keys implement a trait in
/// `cognomen` (`Reason`, `Blurb`, `Hint`, `Help`). Every extra returns
/// [`cognomen::Formatted`].
///
/// ```ignore
/// #[derive(Cognomen)]
/// #[cognomen(lower)]
/// enum SourceKind {
///     #[cognomen(blurb = "microphone / input device")]
///     Mic,
///     App, // App.blurb() == "app" (`as_str`)
/// }
/// ```
///
/// On the enum, `name = "..."` is the default for omitted variants.
/// `name()` means `name = ""`. If the enum does not set a default, omitted
/// variants use `as_str()` / `label()`. Extras are not used for parse or
/// serde.
///
/// # Generated items
///
/// All of these are trait impls. Nothing is inherent on `E`.
///
/// - `cognomen::Label`: `label` / `as_str` / `in_case`
/// - `cognomen::Reason` / `Blurb` / `Hint` / `Help` / `Extra`: extras
/// - `cognomen::Variants` (non-generic, fieldless): `VARIANTS` / `LABELS`
/// - `AsRef<str>`, `PartialEq<str>` (compares the label, not an alias)
/// - `TryFrom<&str>`, `FromStr`, `cognomen::FromLabel` (fieldless enums;
///   `alias` and optional `unknown` fallback)
/// - `Serialize` / `Deserialize` (feature `serde`; fieldless enums)
///
/// See the [`cognomen`](https://docs.rs/cognomen) crate docs for case styles,
/// extras, features, and `no_std`.
#[proc_macro_derive(Cognomen, attributes(cognomen))]
pub fn derive_cognomen(input: TokenStream) -> TokenStream {
    cognomen::derive(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
