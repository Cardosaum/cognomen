//! Compile-time string labels for enum variants.
//!
//! A *cognomen* is an extra name. This crate gives each variant a second,
//! stable label: a `&'static str` whose case you pick in the derive attribute.
//! Conversion runs in the proc-macro. Calls are a `match` on a literal.
//!
//! Downstream crates use this as the seam between a Rust ident and the string
//! a config file, log line, CLI flag, or wire protocol actually carries.
//!
//! Labels and extras are **trait items** in this crate, not inherent methods
//! on `E`. Import [`Label`], [`Reason`], [`Blurb`], ... to call them, or use
//! UFCS (`<E as Label>::as_str(&e)`). A user `fn reason()` or a parent trait
//! of the same name still compiles.
//!
//! # Quick start
//!
//! ```
//! use cognomen::{Case, Cognomen, FromLabel, Label, Variants};
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
//! assert_eq!(Mode::SingleProcess.in_case(Case::Kebab), "single-process");
//! assert_eq!(Mode::try_from("single-process"), Ok(Mode::SingleProcess));
//! assert_eq!("multi_process".parse::<Mode>(), Ok(Mode::MultiProcess));
//! assert!(Mode::SingleProcess == "single-process");
//! assert_eq!(Mode::VARIANTS.len(), 2);
//! ```
//!
//! The first case in `#[cognomen(...)]` is the default (`label()` / `as_str()` /
//! serde out). [`Label::in_case`] converts from the ident in any [`Case`].
//! `PartialEq<str>` on `E` compares the **label**, not an extra.
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
//! - One or more cases, comma-separated. First is the default. Listed cases
//!   are accepted by parse / serde-in; [`Label::in_case`] can still produce
//!   any [`Case`].
//! - `prefix = "cfg"`: accepted for compatibility; case accessors live on
//!   [`Label`], so the prefix no longer names methods.
//! - `crate = ::other::cognomen`: path used in generated code when this crate
//!   is re-exported under another name.
//!
//! **Variant** (optional):
//!
//! - `#[cognomen(rename = "io_error")]`: sets the default label to that
//!   exact string and accepts it when parsing. [`Label::in_case`] still
//!   converts from the ident.
//! - `#[cognomen(alias = "main")]`: extra parse-in string. Does not change
//!   `label()`, `as_str()`, serde-out, [`PartialEq<str>`], or `in_case`.
//!   Repeat the key for more than one alias. Empty `""` is a compile error.
//! - `#[cognomen(unknown)]`: unmatched parse and serde-in become this unit
//!   variant. Fieldless enums only; exactly one such variant. The unmatched
//!   string is not stored. clap `value_parser` still rejects unmatched
//!   input: unknown is a wire fallback, not a flag fallback. Do not default
//!   a catch-all; mark it explicitly.
//!
//! ```
//! use cognomen::{Case, Cognomen, FromLabel, Label};
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
//! assert_eq!(Wire::IoFailed.in_case(Case::Snake), "io_failed");
//! assert_eq!(Wire::from_label("io_error").unwrap(), Wire::IoFailed);
//! assert_eq!(Wire::try_from("io-failed"), Ok(Wire::IoFailed));
//! ```
//!
//! An alias is parse-only. `""` is not an alias; keep that mapping local.
//!
//! ```
//! use cognomen::{Cognomen, FromLabel, Label};
//!
//! #[derive(Debug, PartialEq, Cognomen)]
//! #[cognomen(snake_case)]
//! enum ProcessRole {
//!     #[cognomen(alias = "main")]
//!     Supervisor,
//!     Worker,
//! }
//!
//! assert_eq!(ProcessRole::Supervisor.label(), "supervisor");
//! assert_eq!(ProcessRole::from_label("main").unwrap(), ProcessRole::Supervisor);
//! assert_eq!(ProcessRole::from_label("supervisor").unwrap(), ProcessRole::Supervisor);
//! assert!(ProcessRole::from_label("").is_err());
//! assert!(ProcessRole::Supervisor != "main");
//! ```
//!
//! An unknown variant receives unmatched wire tags. `from_label` is no
//! longer a bijection.
//!
//! ```
//! use cognomen::{Cognomen, FromLabel, Label};
//!
//! #[derive(Debug, PartialEq, Cognomen)]
//! #[cognomen(snake_case)]
//! enum ChannelKind {
//!     Trades,
//!     L2Book,
//!     #[cognomen(unknown)]
//!     Other,
//! }
//!
//! assert_eq!(ChannelKind::from_label("trades").unwrap(), ChannelKind::Trades);
//! assert_eq!(ChannelKind::from_label("nope").unwrap(), ChannelKind::Other);
//! assert_eq!(ChannelKind::Other.label(), "other");
//! ```
//!
//! Any other `name = "..."` is an [extra](#extras).
//!
//! Violations (non-enum, missing case, collisions, empty alias, more than
//! one `unknown`, bad prefix, bad extra, unknown `{field}` placeholder) are
//! compile errors. Variants named `Error`
//! or `Err` are fine: generated `TryFrom` / `FromStr` name [`FromLabelError`]
//! instead of `Self::Error` / `Self::Err`.
//!
//! # Extras
//!
//! Any `name = "..."` in `#[cognomen(...)]` besides `prefix`, `crate`,
//! `rename`, and `alias` is an extra. Known keys (`reason`, `blurb`, `hint`, `help`)
//! implement a trait in this crate ([`Reason`], [`Blurb`], [`Hint`],
//! [`Help`]). Other keys implement [`Extra`] with a private key type.
//!
//! On a variant, that string is the variant's value. On the enum, that string
//! is the default for variants that omit the key. If the enum does not set a
//! default, omitted variants use `as_str()` / `label()` (including `rename`).
//! `name()` on the enum is the same as `name = ""`.
//!
//! Every extra returns [`Formatted`]. Static text is a single literal;
//! `{field}` on a **variant** extra appends that named (or tuple-index)
//! payload. Adding a placeholder does not change the signature. Enum-level
//! defaults cannot contain placeholders. `{` / `}` in the text are written
//! `{{` / `}}`.
//!
//! ```
//! use cognomen::{Blurb, Cognomen, Label};
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
//! use cognomen::{Blurb, Cognomen, Hint};
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
//! Fielded variants keep [`Label`]. Parse, [`Variants`], and serde-in are
//! omitted: a label is not enough to build a payload. Interpolate extras
//! from the fields:
//!
//! ```
//! use cognomen::{Cognomen, Label, Reason};
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
//! A unit extra and a fielded extra with the same key share one trait and
//! one return type:
//!
//! ```
//! use cognomen::{Cognomen, Formatted, Reason};
//!
//! #[derive(Cognomen)]
//! #[cognomen(snake_case)]
//! enum Kind {
//!     #[cognomen(reason = "not implemented")]
//!     Unimplemented,
//! }
//!
//! #[derive(Cognomen)]
//! #[cognomen(snake_case)]
//! enum HostError {
//!     #[cognomen(reason = "host open failed {cause}")]
//!     OpenFailed { cause: &'static str },
//! }
//!
//! fn sentence(e: &impl Reason) -> Formatted<'_> {
//!     e.reason()
//! }
//!
//! assert!(sentence(&HostError::OpenFailed { cause: "busy" }) == "host open failed busy");
//! assert!(sentence(&Kind::Unimplemented) == "not implemented");
//! ```
//!
//! Several extras can coexist. They are not accepted by [`FromLabel`] or
//! serde. Names that collide with generated items (`label`, `as_str`, ...)
//! are compile errors.
//!
//! # Generated API
//!
//! For `#[cognomen(snake_case, kebab-case)]` on `E`:
//!
//! - [`Label`]: `label()` / `as_str()` (default case, or `rename`) and
//!   `in_case(Case)`
//! - [`Reason`] / [`Blurb`] / [`Hint`] / [`Help`] / [`Extra`]: each extra
//!   returns [`Formatted`]. Omitted variants use `as_str()` unless the enum
//!   sets a default
//! - [`Variants`] (non-generic, fieldless enums): `E::VARIANTS` and
//!   `E::LABELS` after `use cognomen::Variants`. Trait items, so they cannot
//!   clash with another derive or a user `const VARIANTS`
//! - No `Display` / `Error` impl on `E`. Print the label with `e.label()`,
//!   or an extra with `write!(f, "{}", e.reason())`
//! - `AsRef<str>`, `PartialEq<str>` / `PartialEq<&str>` (label, not extra or alias)
//! - `TryFrom<&str>`, `FromStr`, [`FromLabel`] (fieldless enums): declared
//!   cases, `rename`, and `alias`; unmatched input errors unless a variant
//!   is marked `unknown`
//! - `Serialize` / `Deserialize` (feature `serde`, fieldless): out is
//!   `label()`, in accepts any declared case, `rename`, or `alias`, and an
//!   `unknown` variant if marked
//! - [`clap::ArgType::value_parser`] (feature `clap`): clap flag parser in
//!   the binary; the enum crate can stay `no_std`. Accepts declared cases,
//!   `rename`, and `alias`. Does not follow `unknown`
//!
//! Nothing is inherent on `E`. A follow-up in `numbered` should move
//! `number()` onto a trait the same way.
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
//! cognomen = { version = "0.5", default-features = false }
//! ```
//!
//! Labels, extras ([`Formatted`]), parse, `AsRef`, and [`Variants`] use only
//! `core`. Add `features = ["alloc"]` to keep the unmatched string on parse
//! errors. Add `features = ["serde"]` for wire formats. Add
//! `features = ["clap"]` in the binary that owns the clap surface, not in a
//! `no_std` kernel.
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

mod extra;
mod formatted;
mod label;

#[doc(inline)]
pub use cognomen_macros::Cognomen;
pub use extra::{Blurb, BlurbKey, Extra, Help, HelpKey, Hint, HintKey, Reason, ReasonKey};
pub use formatted::Formatted;
#[doc(hidden)]
pub use label::__FromDeclared;
pub use label::{Case, FromLabel, Label};

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
/// and [`FromLabel::from_label`]. With `alloc`, the unmatched string is stored
/// in [`Self::input`]. With `std`, this implements [`std::error::Error`].
///
/// ```
/// use cognomen::{Cognomen, FromLabel, FromLabelError};
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
        assert_eq!(e.in_case(Case::Snake), "open_failed");
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

    #[derive(Cognomen)]
    #[cognomen(snake_case)]
    enum KindUnimplemented {
        #[cognomen(reason = "not implemented")]
        Unimplemented,
    }

    impl KindUnimplemented {
        fn reason(&self) -> u8 {
            0
        }

        fn as_str(&self) -> u8 {
            1
        }
    }

    #[test]
    fn extras_share_one_trait_and_do_not_clash() {
        fn sentence(e: &impl Reason) -> Formatted<'_> {
            e.reason()
        }
        assert!(sentence(&HostError::OpenFailed { cause: "busy" }) == "host open failed busy");
        assert!(sentence(&KindUnimplemented::Unimplemented) == "not implemented");
        assert_eq!(KindUnimplemented::Unimplemented.reason(), 0);
        assert_eq!(KindUnimplemented::Unimplemented.as_str(), 1);
        assert_eq!(
            <KindUnimplemented as Reason>::reason(&KindUnimplemented::Unimplemented),
            "not implemented"
        );
        assert_eq!(
            <KindUnimplemented as Label>::as_str(&KindUnimplemented::Unimplemented),
            "unimplemented"
        );
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
        assert_eq!(Mode::SingleProcess.in_case(Case::Kebab), "single-process");
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case)]
    enum ProcessRole {
        #[cognomen(alias = "main")]
        Supervisor,
        Worker,
    }

    #[test]
    fn parse_alias_is_parse_only() {
        assert_eq!(ProcessRole::Supervisor.label(), "supervisor");
        assert_eq!(ProcessRole::Supervisor.as_str(), "supervisor");
        assert_eq!(ProcessRole::Supervisor.in_case(Case::Snake), "supervisor");
        assert_eq!(ProcessRole::Supervisor.in_case(Case::Kebab), "supervisor");
        assert_eq!(ProcessRole::LABELS, &["supervisor", "worker"]);
        assert_eq!(
            ProcessRole::from_label("main").unwrap(),
            ProcessRole::Supervisor
        );
        assert_eq!(
            ProcessRole::from_label("supervisor").unwrap(),
            ProcessRole::Supervisor
        );
        assert_eq!(
            "main".parse::<ProcessRole>().unwrap(),
            ProcessRole::Supervisor
        );
        assert!(ProcessRole::from_label("").is_err());
        assert!(ProcessRole::Supervisor == "supervisor");
        assert!(ProcessRole::Supervisor != "main");
        assert!("main" != ProcessRole::Supervisor);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case)]
    enum ChannelKind {
        Trades,
        L2Book,
        #[cognomen(unknown)]
        Other,
    }

    #[test]
    fn unknown_variant_is_parse_fallback() {
        assert_eq!(
            ChannelKind::from_label("trades").unwrap(),
            ChannelKind::Trades
        );
        assert_eq!(
            ChannelKind::from_label("l2_book").unwrap(),
            ChannelKind::L2Book
        );
        assert_eq!(ChannelKind::from_label("nope").unwrap(), ChannelKind::Other);
        assert_eq!(ChannelKind::from_label("").unwrap(), ChannelKind::Other);
        assert_eq!(
            "mystery".parse::<ChannelKind>().unwrap(),
            ChannelKind::Other
        );
        assert_eq!(ChannelKind::Other.label(), "other");
        assert_eq!(ChannelKind::Other.as_str(), "other");
        assert!(ChannelKind::Other == "other");
        assert!(ChannelKind::Other != "nope");
        assert_eq!(ChannelKind::try_from("nope"), Ok(ChannelKind::Other));
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
    #[cognomen(snake_case, kebab-case)]
    enum MultiAlias {
        #[cognomen(alias = "io", alias = "i/o")]
        IoFailed,
        OpenFailed,
    }

    #[test]
    fn multiple_aliases_and_declared_cases() {
        assert_eq!(MultiAlias::IoFailed.label(), "io_failed");
        assert_eq!(MultiAlias::from_label("io").unwrap(), MultiAlias::IoFailed);
        assert_eq!(MultiAlias::from_label("i/o").unwrap(), MultiAlias::IoFailed);
        assert_eq!(
            MultiAlias::from_label("io-failed").unwrap(),
            MultiAlias::IoFailed
        );
        assert!(MultiAlias::IoFailed != "io");
        assert!(MultiAlias::IoFailed == "io-failed");
        assert!(MultiAlias::from_label("nope").is_err());
    }

    const fn mode_label_in_const() -> &'static str {
        Mode::LABELS[0]
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
        assert_eq!(Alias::SingleProcess.in_case(Case::Snake), "single_process");
        assert_eq!(Alias::SingleProcess.in_case(Case::Kebab), "single-process");
        assert_eq!(Alias::SingleProcess.in_case(Case::Camel), "singleProcess");
        assert_eq!(Alias::SingleProcess.in_case(Case::Pascal), "SingleProcess");
        assert_eq!(
            Alias::SingleProcess.in_case(Case::ScreamingSnake),
            "SINGLE_PROCESS"
        );
        assert_eq!(Alias::SingleProcess.in_case(Case::Lower), "singleprocess");
        assert_eq!(Alias::SingleProcess.in_case(Case::Upper), "SINGLEPROCESS");
        assert_eq!(Alias::SingleProcess.in_case(Case::Title), "Single Process");
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
        assert_eq!(
            Prefixed::EnableLogging.in_case(Case::Snake),
            "enable_logging"
        );
        assert_eq!(
            Prefixed::EnableLogging.in_case(Case::Kebab),
            "enable-logging"
        );
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
        let alias_in: ProcessRole = serde_json::from_str("\"main\"").unwrap();
        assert_eq!(alias_in, ProcessRole::Supervisor);
        assert_eq!(
            serde_json::to_string(&ProcessRole::Supervisor).unwrap(),
            "\"supervisor\""
        );
        let unknown_in: ChannelKind = serde_json::from_str("\"nope\"").unwrap();
        assert_eq!(unknown_in, ChannelKind::Other);
        assert_eq!(
            serde_json::to_string(&ChannelKind::Other).unwrap(),
            "\"other\""
        );
        let generic = Generic::<3>::LeftHand;
        let gs = serde_json::to_string(&generic).unwrap();
        assert_eq!(gs, "\"left_hand\"");
        let gback: Generic<3> = serde_json::from_str(&gs).unwrap();
        assert_eq!(gback, generic);
    }
}
