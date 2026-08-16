//! Proc-macro implementation for [`cognomen`](https://docs.rs/cognomen).
//!
//! Depend on the `cognomen` crate, not this one. This crate is the host
//! proc-macro; generated items use `core` so the runtime can be `no_std`.

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
/// - `no_display`: do not implement `Display`. Use this when another derive
///   on the same type already implements it (for example numbered).
/// - `no_variants`: do not emit `VARIANTS`. `LABELS` is still emitted.
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
/// # Extra methods
///
/// Any other `name = "..."` besides `no_display` / `no_variants` becomes
/// `const fn name(&self) -> &'static str`.
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
/// variants use `as_str()` / `label()`. Extra methods are not used for
/// parse, `Display`, or serde.
///
/// # Generated items
///
/// - `const fn label(&self) -> &'static str`
/// - `const fn as_str(&self) -> &'static str`
/// - `const fn {prefix}_{case}(&self) -> &'static str` for each declared case
/// - `const fn {name}(&self) -> &'static str` for each extra
/// - `VARIANTS` / `LABELS` (non-generic enums; `no_variants` skips `VARIANTS`)
/// - `Display` (skipped by `no_display`), `AsRef<str>`, `PartialEq<str>`
/// - `TryFrom<&str>`, `FromStr`, `from_label`
/// - `Serialize` / `Deserialize` (feature `serde`)
///
/// See the [`cognomen`](https://docs.rs/cognomen) crate docs for case styles,
/// extra methods, features, and `no_std`.
#[proc_macro_derive(Cognomen, attributes(cognomen))]
pub fn derive_cognomen(input: TokenStream) -> TokenStream {
    cognomen::derive(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
