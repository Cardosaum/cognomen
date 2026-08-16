//! Compile-time string labels for unit-like enum variants.
//!
//! A *cognomen* is an extra name. This crate gives each variant a second,
//! stable label: a `&'static str` whose case you pick in the derive attribute.
//! Conversion runs in the proc-macro. Calls are a `match` on a literal.
//!
//! Downstream crates use this as the seam between a Rust ident and the string
//! a config file, log line, CLI flag, or wire protocol actually carries.
//!
//! # Quick start
//!
//! ```
//! use cognomen::Cognomen;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
//! #[cognomen(snake_case, kebab-case)]
//! enum Mode {
//!     SingleProcess,
//!     MultiProcess,
//! }
//!
//! assert_eq!(Mode::SingleProcess.label(), "single_process");
//! assert_eq!(Mode::MultiProcess.as_str(), "multi_process");
//! assert_eq!(Mode::SingleProcess.label_kebab(), "single-process");
//! assert_eq!(Mode::try_from("single-process"), Ok(Mode::SingleProcess));
//! assert_eq!("multi_process".parse::<Mode>(), Ok(Mode::MultiProcess));
//! assert!(Mode::SingleProcess == "single-process");
//! assert_eq!(Mode::VARIANTS.len(), 2);
//! ```
//!
//! The first case in `#[cognomen(...)]` is the default (`label()` / `as_str()` /
//! `Display` / serde out). Every listed case also gets `{prefix}_{case}`.
//!
//! # Case styles
//!
//! | Attribute | Aliases | `VariantName` becomes |
//! |-----------|---------|-----------------------|
//! | `snake_case` | `snake` | `variant_name` |
//! | `kebab-case` | `kebab_case`, `kebab` | `variant-name` |
//! | `camelCase` | `camel_case`, `camel` | `variantName` |
//! | `PascalCase` | `pascal_case`, `pascal` | `VariantName` |
//! | `SCREAMING_SNAKE_CASE` | `screaming_snake_case`, `screaming` | `VARIANT_NAME` |
//! | `lower` | `lowercase` | `variantname` |
//! | `upper` | `uppercase` | `VARIANTNAME` |
//! | `title` | `title_case` | `Variant Name` |
//!
//! # Attributes
//!
//! **Container** (required): `#[cognomen(<case>, ...)]`
//!
//! - One or more cases, comma-separated. First is the default.
//! - `prefix = "cfg"`: accessor names become `cfg_snake`, `cfg_kebab`, ...
//!   (must be a non-empty ASCII identifier; default `label`).
//! - `crate = ::other::cognomen`: path used in generated code when this crate
//!   is re-exported under another name.
//! - `no_display`: do not implement `Display`. Use this when another derive
//!   on the same type already implements it (for example numbered).
//! - `no_variants`: do not emit `VARIANTS`. `LABELS` is still emitted.
//!
//! **Variant** (optional): `#[cognomen(rename = "io_error")]`
//!
//! Sets the default label to that exact string and accepts it when parsing.
//! Other case accessors still convert from the ident.
//!
//! ```
//! use cognomen::Cognomen;
//!
//! #[derive(Debug, PartialEq, Cognomen)]
//! #[cognomen(snake_case, kebab-case)]
//! enum Wire {
//!     #[cognomen(rename = "io_error")]
//!     IoFailed,
//!     OpenFailed,
//! }
//!
//! assert_eq!(Wire::IoFailed.label(), "io_error");
//! assert_eq!(Wire::IoFailed.label_snake(), "io_failed");
//! assert_eq!(Wire::from_label("io_error").unwrap(), Wire::IoFailed);
//! assert_eq!(Wire::try_from("io-failed"), Ok(Wire::IoFailed));
//! ```
//!
//! Any other `name = "..."` is an [extra method](#extra-methods).
//!
//! Violations (non-enum, fields, missing case, collisions, bad prefix, bad
//! extra) are compile errors. Variants named `Error` or `Err` are fine:
//! generated `TryFrom` / `FromStr` name [`FromLabelError`] instead of
//! `Self::Error` / `Self::Err`.
//!
//! # Extra methods
//!
//! Any `name = "..."` in `#[cognomen(...)]` besides `prefix`, `crate`,
//! `rename`, `no_display`, and `no_variants` becomes
//! `const fn name(&self) -> &'static str`.
//!
//! On a variant, that string is the variant's value. On the enum, that string
//! is the default for variants that omit the key. If the enum does not set a
//! default, omitted variants use `as_str()` / `label()` (including `rename`).
//! `name()` on the enum is the same as `name = ""`.
//!
//! ```
//! use cognomen::Cognomen;
//!
//! #[derive(Cognomen)]
//! #[cognomen(lower)]
//! enum SourceKind {
//!     #[cognomen(blurb = "microphone / input device")]
//!     Mic,
//!     #[cognomen(blurb = "system-wide loopback")]
//!     System,
//!     App,
//! }
//!
//! assert_eq!(SourceKind::Mic.as_str(), "mic");
//! assert_eq!(SourceKind::Mic.blurb(), "microphone / input device");
//! assert_eq!(SourceKind::App.blurb(), "app");
//! ```
//!
//! An enum-level default overrides `as_str()` for omitted variants:
//!
//! ```
//! use cognomen::Cognomen;
//!
//! #[derive(Cognomen)]
//! #[cognomen(lower, blurb = "", hint = "n/a")]
//! enum SourceKind {
//!     #[cognomen(blurb = "microphone / input device", hint = "CoreAudio input")]
//!     Mic,
//!     App,
//! }
//!
//! assert_eq!(SourceKind::Mic.blurb(), "microphone / input device");
//! assert_eq!(SourceKind::App.blurb(), "");
//! assert_eq!(SourceKind::Mic.hint(), "CoreAudio input");
//! assert_eq!(SourceKind::App.hint(), "n/a");
//! ```
//!
//! Several extras can coexist. They are not accepted by `from_label`,
//! `Display`, or serde. Names that collide with generated items (`label`,
//! `as_str`, `{prefix}_{case}`, ...) are compile errors.
//!
//! # Generated API
//!
//! For `#[cognomen(snake_case, kebab-case)]` on `E`:
//!
//! - `label()` / `as_str()` -> `&'static str` (default case, or `rename`)
//! - `label_snake()`, `label_kebab()`, ...
//! - `{name}()` for each extra (`blurb()`, `hint()`, ...); omitted variants
//!   use `as_str()` unless the enum sets a default
//! - `E::VARIANTS: &'static [E]` and `E::LABELS: &'static [&'static str]`
//!   (`no_variants` skips `VARIANTS`)
//! - `Display` (`no_display` skips this), `AsRef<str>`, `PartialEq<str>` /
//!   `PartialEq<&str>`
//! - `TryFrom<&str>`, `FromStr`, `E::from_label`
//! - `Serialize` / `Deserialize` (feature `serde`): out is `label()`, in
//!   accepts any declared case or `rename`
//!
//! # Features
//!
//! | Feature | Default | Unlocks |
//! |---------|---------|---------|
//! | `std` | yes | `alloc` + [`std::error::Error`] for [`FromLabelError`] |
//! | `alloc` | via `std` | [`FromLabelError::input`] stores the unmatched string |
//! | `serde` | no | `Serialize` / `Deserialize` |
//!
//! # `no_std`
//!
//! ```toml
//! cognomen = { version = "0.1", default-features = false }
//! ```
//!
//! Labels, parse, `Display`, `AsRef`, and `VARIANTS` use only `core`. Add
//! `features = ["alloc"]` to keep the unmatched string on parse errors. Add
//! `features = ["serde"]` for wire formats.
//!
//! # Word splitting
//!
//! Idents are split on ASCII camel-case boundaries. Acronyms stay together
//! (`HTTPResponse` -> `http_response`). Digits stay glued (`Utf8` -> `utf8`,
//! `IPv4` -> `i_pv4`). Re-Pascal of an acronym title-cases the run
//! (`HTTPResponse` -> `HttpResponse`).
//!
//! # MSRV
//!
//! Rust 1.71.1.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate self as cognomen;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[doc(inline)]
pub use cognomen_macros::Cognomen;

/// Error returned when a string matches no declared label.
///
/// Produced by [`TryFrom<&str>`](core::convert::TryFrom), [`core::str::FromStr`],
/// and `E::from_label`. With `alloc`, the unmatched string is stored in
/// [`Self::input`]. With `std`, this implements [`std::error::Error`].
///
/// ```
/// use cognomen::{Cognomen, FromLabelError};
///
/// #[derive(Debug, Cognomen)]
/// #[cognomen(snake_case)]
/// enum Mode {
///     SingleProcess,
/// }
///
/// let err = Mode::from_label("nope").unwrap_err();
/// # #[cfg(feature = "alloc")]
/// assert_eq!(err.input, "nope");
/// # #[cfg(feature = "alloc")]
/// assert!(err.to_string().contains("nope"));
/// let _: FromLabelError = err;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FromLabelError {
    /// The string that did not match.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub input: alloc::string::String,
}

impl FromLabelError {
    /// Build an error for the unmatched `input` string.
    ///
    /// With `alloc`, that string is stored in [`Self::input`].
    #[must_use]
    pub fn new(input: &str) -> Self {
        #[cfg(not(feature = "alloc"))]
        let _ = input;
        Self {
            #[cfg(feature = "alloc")]
            input: alloc::string::String::from(input),
        }
    }
}

impl core::fmt::Display for FromLabelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "alloc")]
        {
            write!(f, "no cognomen label matches `{}`", self.input)
        }
        #[cfg(not(feature = "alloc"))]
        {
            f.write_str("no cognomen label matches")
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for FromLabelError {}

#[cfg(feature = "serde")]
#[doc(hidden)]
pub use serde as __serde;

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case, kebab-case)]
    enum Mode {
        SingleProcess,
        MultiProcess,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(lower)]
    enum Kind {
        #[cognomen(blurb = "microphone / input device")]
        Mic,
        #[cognomen(blurb = "system-wide loopback", hint = "loopback")]
        System,
        App,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(lower, blurb = "", hint = "")]
    enum KindEmpty {
        #[cognomen(blurb = "microphone / input device")]
        Mic,
        App,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(lower, no_display, no_variants)]
    enum Quiet {
        Mic,
        App,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(lower)]
    enum Status {
        Error,
        Err,
        Ok,
    }

    #[test]
    fn error_and_err_variants() {
        assert_eq!(Status::Error.as_str(), "error");
        assert_eq!(Status::try_from("err"), Ok(Status::Err));
        assert_eq!("ok".parse::<Status>().unwrap(), Status::Ok);
    }

    #[test]
    fn skip_display_and_variants() {
        assert_eq!(Quiet::Mic.as_str(), "mic");
        assert_eq!(Quiet::App.label(), "app");
        assert_eq!(Quiet::LABELS, &["mic", "app"]);
        assert_eq!(Quiet::from_label("mic").unwrap(), Quiet::Mic);
    }

    #[test]
    fn extra_methods() {
        assert_eq!(Kind::Mic.as_str(), "mic");
        assert_eq!(Kind::Mic.blurb(), "microphone / input device");
        assert_eq!(Kind::System.blurb(), "system-wide loopback");
        assert_eq!(Kind::App.blurb(), "app");
        assert_eq!(Kind::Mic.hint(), "mic");
        assert_eq!(Kind::System.hint(), "loopback");
        assert_eq!(Kind::App.hint(), "app");
        assert_eq!(KindEmpty::Mic.blurb(), "microphone / input device");
        assert_eq!(KindEmpty::App.blurb(), "");
        assert_eq!(KindEmpty::Mic.hint(), "");
        assert_eq!(KindEmpty::App.hint(), "");
    }

    #[test]
    fn aliases_and_tables() {
        assert_eq!(Mode::SingleProcess.label(), "single_process");
        assert_eq!(Mode::MultiProcess.as_str(), "multi_process");
        assert_eq!(Mode::SingleProcess.label_kebab(), "single-process");
        assert_eq!(
            core::convert::AsRef::<str>::as_ref(&Mode::SingleProcess),
            "single_process"
        );
        assert_eq!(Mode::VARIANTS, &[Mode::SingleProcess, Mode::MultiProcess]);
        assert_eq!(Mode::LABELS, &["single_process", "multi_process"]);
        assert!(Mode::SingleProcess == "single_process");
        assert!(Mode::SingleProcess == "single-process");
        assert!("multi-process" == Mode::MultiProcess);
    }

    #[test]
    fn parse() {
        assert_eq!(
            Mode::from_label("single-process").unwrap(),
            Mode::SingleProcess
        );
        assert!(Mode::try_from("nope").is_err());
        assert_eq!("multi_process".parse::<Mode>().unwrap(), Mode::MultiProcess);
        #[cfg(feature = "alloc")]
        assert_eq!(Mode::try_from("nope").unwrap_err().input, "nope");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip() {
        let v = Mode::SingleProcess;
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, "\"single_process\"");
        let back: Mode = serde_json::from_str(&s).unwrap();
        assert_eq!(back, v);
        let kebab: Mode = serde_json::from_str("\"single-process\"").unwrap();
        assert_eq!(kebab, v);
    }
}
