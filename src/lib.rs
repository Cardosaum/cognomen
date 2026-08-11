//! # cognomen
//!
//! *Cognomen* is Latin for "an extra name given to a person or thing". It is a
//! proc-macro derive that gives every unit-like variant of an enum a second
//! name: a stable, case-configured string label exposed via [`label`](Cognomen).
//!
//! Case conversion runs at compile time and is emitted as a `&'static str`.
//!
//! ```rust
//! use cognomen::Cognomen;
//!
//! // First case is the default (`label()`); each case gets `<prefix>_<case>`.
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
//! #[cognomen(snake_case, kebab-case)]
//! enum Mode {
//!     SingleProcess, // "single_process" / "single-process"
//!     MultiProcess,  // "multi_process"  / "multi-process"
//! }
//!
//! assert_eq!(Mode::SingleProcess.label(), "single_process");
//! assert_eq!(Mode::SingleProcess.label_kebab(), "single-process");
//! assert_eq!(Mode::try_from("single-process"), Ok(Mode::SingleProcess));
//! assert_eq!("multi_process".parse::<Mode>(), Ok(Mode::MultiProcess));
//! ```
//!
//! See the crate README for case styles (including short aliases), `prefix`,
//! and requirements. Failures are compile-time errors pinned by trybuild UI
//! tests under `tests/ui/`.

mod cognomen;

use proc_macro::TokenStream;

/// Derive `label` and a `<prefix>_<case>` accessor for every declared case on
/// a unit-like enum.
///
/// Required container attribute: `#[cognomen(<case>, ...)]`. The first case is
/// the default for `label()`. Optional `prefix = "..."` (default `"label"`).
///
/// Also implements `TryFrom<&str>` and `FromStr` for every declared case.
#[proc_macro_derive(Cognomen, attributes(cognomen))]
pub fn derive_cognomen(input: TokenStream) -> TokenStream {
    cognomen::derive(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
