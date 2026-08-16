//! Compile-time string labels for enum variants.
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
//! use cognomen::{Cognomen, Variants};
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
//! serde out). Every listed case also gets `{prefix}_{case}`.
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
//! Violations (non-enum, missing case, collisions, bad prefix, bad extra,
//! unknown `{field}` placeholder) are compile errors. Variants named `Error`
//! or `Err` are fine: generated `TryFrom` / `FromStr` name [`FromLabelError`]
//! instead of `Self::Error` / `Self::Err`.
//!
//! # Extra methods
//!
//! Any `name = "..."` in `#[cognomen(...)]` besides `prefix`, `crate`, and
//! `rename` becomes an extra method.
//!
//! On a variant, that string is the variant's value. On the enum, that string
//! is the default for variants that omit the key. If the enum does not set a
//! default, omitted variants use `as_str()` / `label()` (including `rename`).
//! `name()` on the enum is the same as `name = ""`.
//!
//! `{field}` in a **variant** extra interpolates that named (or tuple-index)
//! payload. The method then returns [`Formatted`] instead of `&'static str`.
//! Enum-level defaults cannot contain placeholders. `{` / `}` in the text
//! are written `{{` / `}}`.
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
//! Fielded variants keep `label()` / `as_str()` (the ident case). Parse,
//! [`Variants`], and serde are omitted: a label is not enough to build a
//! payload. Interpolate extras from the fields:
//!
//! ```
//! use cognomen::Cognomen;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Cognomen)]
//! #[cognomen(snake_case)]
//! enum HostError {
//!     #[cognomen(reason = "host backend unsupported {capability}")]
//!     Unsupported { capability: &'static str },
//!     #[cognomen(reason = "host open failed {cause}")]
//!     OpenFailed { cause: &'static str },
//!     #[cognomen(reason = "host refused request {why}")]
//!     BadRequest { why: &'static str },
//!     #[cognomen(reason = "host io failed {status}")]
//!     Io { status: &'static str },
//! }
//!
//! let e = HostError::OpenFailed { cause: "busy" };
//! assert_eq!(e.as_str(), "open_failed");
//! assert_eq!(e.reason(), "host open failed busy");
//! assert_eq!(format!("{}", e.reason()), "host open failed busy");
//! ```
//!
//! Several extras can coexist. They are not accepted by `from_label` or
//! serde. Names that collide with generated items (`label`, `as_str`,
//! `{prefix}_{case}`, ...) are compile errors.
//!
//! # Generated API
//!
//! For `#[cognomen(snake_case, kebab-case)]` on `E`:
//!
//! - `label()` / `as_str()` -> `&'static str` (default case, or `rename`)
//! - `label_snake()`, `label_kebab()`, ...
//! - `{name}()` for each extra (`blurb()`, `hint()`, `reason()`, ...); omitted
//!   variants use `as_str()` unless the enum sets a default. `{field}`
//!   interpolation returns [`Formatted`]
//! - [`Variants`] (non-generic, fieldless enums): `E::VARIANTS` and
//!   `E::LABELS` after `use cognomen::Variants`. These are trait items, so
//!   they cannot clash with another derive or a user `const VARIANTS`.
//! - No `Display` impl. Print the label with `e.label()` / `e.as_str()`,
//!   or an interpolating extra with `write!(f, "{}", e.reason())`.
//! - `AsRef<str>`, `PartialEq<str>` / `PartialEq<&str>`
//! - `TryFrom<&str>`, `FromStr`, `E::from_label` (fieldless enums)
//! - `Serialize` / `Deserialize` (feature `serde`, fieldless): out is
//!   `label()`, in accepts any declared case or `rename`
//! - [`clap::ArgType::value_parser`] (feature `clap`): clap flag parser in
//!   the binary; the enum crate can stay `no_std`
//!
//! # Features
//!
//! | Feature | Default | Unlocks |
//! |---------|---------|---------|
//! | `std` | yes | `alloc` + [`std::error::Error`] for [`FromLabelError`] |
//! | `alloc` | via `std` | [`FromLabelError::input`] stores the unmatched string |
//! | `serde` | no | `Serialize` / `Deserialize` |
//! | `clap` | no | [`clap::ArgType`] (`T::value_parser()`); implies `std` |
//!
//! # `no_std`
//!
//! ```toml
//! cognomen = { version = "0.4", default-features = false }
//! ```
//!
//! Labels, parse, `AsRef`, and [`Variants`] use only `core`. Add
//! `features = ["alloc"]` to keep the unmatched string on parse errors. Add
//! `features = ["serde"]` for wire formats. Add `features = ["clap"]` in the
//! binary that owns the clap surface, not in a `no_std` kernel.
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
//! Rust 1.71.1 for default features, `alloc`, and `serde`. The `clap`
//! feature follows clap's rustc floor (4.6 needs 1.85).

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate self as cognomen;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod formatted;

#[doc(inline)]
pub use cognomen_macros::Cognomen;
pub use formatted::Formatted;

#[cfg(feature = "clap")]
#[cfg_attr(docsrs, doc(cfg(feature = "clap")))]
pub mod clap;

/// Declaration-order tables for a non-generic cognomen enum.
///
/// `VARIANTS` and `LABELS` live on this trait, not as inherent items on
/// `E`. Another derive, or a user `const VARIANTS`, cannot clash with them.
/// Import the trait to use `E::VARIANTS`, or spell the path:
/// `<E as cognomen::Variants>::VARIANTS`.
///
/// ```
/// use cognomen::{Cognomen, Variants};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Cognomen)]
/// #[cognomen(lower)]
/// enum Kind {
///     Mic,
///     App,
/// }
///
/// assert_eq!(Kind::VARIANTS, &[Kind::Mic, Kind::App]);
/// assert_eq!(Kind::LABELS, &["mic", "app"]);
/// ```
pub trait Variants: Sized + 'static {
    /// All variants in declaration order.
    const VARIANTS: &'static [Self];

    /// Default label for each variant in declaration order.
    const LABELS: &'static [&'static str];
}

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

    use std::string::ToString;

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
    #[cognomen(lower)]
    enum Shared {
        Mic,
        App,
    }

    impl core::fmt::Display for Shared {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("user")
        }
    }

    impl Shared {
        pub const VARIANTS: &'static [&'static str] = &["mic", "app"];
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
    fn tables_do_not_clash_with_user_items() {
        assert_eq!(Shared::VARIANTS, &["mic", "app"]);
        assert_eq!(<Shared as Variants>::VARIANTS, &[Shared::Mic, Shared::App]);
        assert_eq!(<Shared as Variants>::LABELS, &["mic", "app"]);
        assert_eq!(Shared::Mic.to_string(), "user");
        assert_eq!(Shared::Mic.as_str(), "mic");
    }

    #[derive(Debug, Clone, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case)]
    enum HostError {
        #[cognomen(reason = "host backend unsupported {capability}")]
        Unsupported { capability: &'static str },
        #[cognomen(reason = "host open failed {cause}")]
        OpenFailed { cause: &'static str },
        #[cognomen(reason = "host refused request {why}")]
        BadRequest { why: &'static str },
        #[cognomen(reason = "host io failed {status}")]
        Io { status: &'static str },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case)]
    enum Mixed {
        #[cognomen(reason = "stdout write failed")]
        Write,
        #[cognomen(reason = "host open failed {cause}")]
        OpenFailed { cause: &'static str },
        #[cognomen(reason = "use {{braces}} {name}")]
        Escaped { name: &'static str },
        #[cognomen(reason = "open failed {0}")]
        Tuple(u8),
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
    fn interpolates_named_payload() {
        let e = HostError::OpenFailed { cause: "busy" };
        assert_eq!(e.as_str(), "open_failed");
        assert_eq!(e.label_snake(), "open_failed");
        assert_eq!(e.reason(), "host open failed busy");
        assert_eq!(
            HostError::Unsupported { capability: "x" }.reason(),
            "host backend unsupported x"
        );
        assert_eq!(
            HostError::BadRequest {
                why: "device name has interior NUL"
            }
            .reason(),
            "host refused request device name has interior NUL"
        );
        assert_eq!(
            HostError::Io {
                status: "short write"
            }
            .reason(),
            "host io failed short write"
        );
        assert!(e == "open_failed");
        assert_eq!(core::convert::AsRef::<str>::as_ref(&e), "open_failed");
    }

    #[test]
    fn interpolates_mixed_and_tuple() {
        assert_eq!(Mixed::Write.reason(), "stdout write failed");
        assert_eq!(
            Mixed::OpenFailed { cause: "busy" }.reason(),
            "host open failed busy"
        );
        assert_eq!(Mixed::Escaped { name: "mic" }.reason(), "use {braces} mic");
        assert_eq!(Mixed::Tuple(7).reason(), "open failed 7");
        assert_eq!(Mixed::Tuple(7).as_str(), "tuple");
    }

    #[cfg(feature = "alloc")]
    #[derive(Debug, Clone, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case)]
    enum OwnedHostError {
        #[cognomen(reason = "host open failed {cause}")]
        OpenFailed { cause: alloc::string::String },
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn interpolates_owned_string() {
        let e = OwnedHostError::OpenFailed {
            cause: alloc::string::String::from("busy"),
        };
        assert_eq!(e.reason(), "host open failed busy");
        assert_eq!(e.reason().to_string(), "host open failed busy");
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

    const fn mode_label_in_const() -> &'static str {
        Mode::SingleProcess.label()
    }

    #[test]
    fn works_in_const() {
        assert_eq!(mode_label_in_const(), "single_process");
        const TABLES: &[&str] = Mode::LABELS;
        assert_eq!(TABLES, &["single_process", "multi_process"]);
    }

    #[test]
    fn from_label_error_new() {
        let err = FromLabelError::new("nope");
        #[cfg(feature = "alloc")]
        {
            assert_eq!(err.input, "nope");
            assert_eq!(err.to_string(), "no cognomen label matches `nope`");
        }
        #[cfg(not(feature = "alloc"))]
        {
            assert_eq!(err.to_string(), "no cognomen label matches");
        }
        #[cfg(feature = "std")]
        {
            let _: &dyn std::error::Error = &err;
        }
    }

    #[test]
    fn partial_eq_mismatch() {
        assert!(Mode::SingleProcess != "nope");
        assert!("nope" != Mode::SingleProcess);
        assert!(Mode::MultiProcess != "single_process");
        assert!(Mode::SingleProcess == "single-process");
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case)]
    enum Acronym {
        HTTPResponse,
        Utf8,
        IPv4,
    }

    #[test]
    fn word_splitting() {
        assert_eq!(Acronym::HTTPResponse.as_str(), "http_response");
        assert_eq!(Acronym::Utf8.as_str(), "utf8");
        assert_eq!(Acronym::IPv4.as_str(), "i_pv4");
        assert_eq!(
            Acronym::from_label("http_response").unwrap(),
            Acronym::HTTPResponse
        );
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(lower, blurb())]
    enum ParenExtra {
        #[cognomen(blurb = "mic")]
        Mic,
        App,
    }

    #[test]
    fn extra_paren_default_is_empty() {
        assert_eq!(ParenExtra::Mic.blurb(), "mic");
        assert_eq!(ParenExtra::App.blurb(), "");
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case)]
    enum Generic<const N: usize> {
        LeftHand,
        RightHand,
    }

    #[test]
    fn generic_has_labels_not_tables() {
        assert_eq!(Generic::<1>::LeftHand.as_str(), "left_hand");
        assert_eq!(
            Generic::<2>::from_label("right_hand").unwrap(),
            Generic::<2>::RightHand
        );
    }

    impl Shared {
        pub const LABELS: &'static [&'static str] = &["user-mic", "user-app"];
    }

    #[test]
    fn user_labels_do_not_hide_trait_labels() {
        assert_eq!(Shared::LABELS, &["user-mic", "user-app"]);
        assert_eq!(<Shared as Variants>::LABELS, &["mic", "app"]);
    }

    #[test]
    fn error_and_err_parse() {
        assert_eq!(Status::from_label("error").unwrap(), Status::Error);
        assert_eq!(Status::from_label("err").unwrap(), Status::Err);
        assert!(Status::from_label("ok").is_ok());
        assert!(Status::try_from("Error").is_err());
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(
        snake, kebab, camel_case, pascal, screaming, lowercase, uppercase, title_case
    )]
    enum Alias {
        SingleProcess,
    }

    #[test]
    fn case_aliases() {
        assert_eq!(Alias::SingleProcess.label(), "single_process");
        assert_eq!(Alias::SingleProcess.label_snake(), "single_process");
        assert_eq!(Alias::SingleProcess.label_kebab(), "single-process");
        assert_eq!(Alias::SingleProcess.label_camel(), "singleProcess");
        assert_eq!(Alias::SingleProcess.label_pascal(), "SingleProcess");
        assert_eq!(
            Alias::SingleProcess.label_screaming_snake(),
            "SINGLE_PROCESS"
        );
        assert_eq!(Alias::SingleProcess.label_lower(), "singleprocess");
        assert_eq!(Alias::SingleProcess.label_upper(), "SINGLEPROCESS");
        assert_eq!(Alias::SingleProcess.label_title(), "Single Process");
        assert_eq!(
            Alias::from_label("single-process").unwrap(),
            Alias::SingleProcess
        );
        assert_eq!(
            Alias::from_label("SINGLE_PROCESS").unwrap(),
            Alias::SingleProcess
        );
        assert_eq!(
            Alias::from_label("Single Process").unwrap(),
            Alias::SingleProcess
        );
        assert!("singleProcess" == Alias::SingleProcess);
        assert!(Alias::SingleProcess == *"single_process");
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case, kebab-case, prefix = "cfg")]
    enum Prefixed {
        EnableLogging,
    }

    #[test]
    fn prefix_and_as_ref() {
        assert_eq!(Prefixed::EnableLogging.label(), "enable_logging");
        assert_eq!(Prefixed::EnableLogging.cfg_snake(), "enable_logging");
        assert_eq!(Prefixed::EnableLogging.cfg_kebab(), "enable-logging");
        assert_eq!(
            core::convert::AsRef::<str>::as_ref(&Prefixed::EnableLogging),
            "enable_logging"
        );
        assert_eq!(
            "enable-logging".parse::<Prefixed>().unwrap(),
            Prefixed::EnableLogging
        );
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
        let generic = Generic::<3>::LeftHand;
        let gs = serde_json::to_string(&generic).unwrap();
        assert_eq!(gs, "\"left_hand\"");
        let gback: Generic<3> = serde_json::from_str(&gs).unwrap();
        assert_eq!(gback, generic);
    }
}
