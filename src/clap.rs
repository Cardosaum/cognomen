//! Clap value parser for cognomen enums.
//!
//! The defining crate stays free of clap: import [`ArgType`] in the binary
//! and call `T::value_parser()`.
//!
//! ```
//! use clap::Parser;
//! use cognomen::clap::ArgType;
//! use cognomen::Cognomen;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
//! #[cognomen(lower)]
//! enum Kind {
//!     Mic,
//!     App,
//! }
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[arg(long, value_parser = Kind::value_parser())]
//!     kind: Kind,
//! }
//!
//! let cli = Cli::try_parse_from(["prog", "--kind", "mic"]).unwrap();
//! assert_eq!(cli.kind, Kind::Mic);
//! ```

use core::marker::PhantomData;
use core::str::FromStr;
use std::boxed::Box;

use ::clap::builder::PossibleValue;
use ::clap::builder::StringValueParser;
use ::clap::builder::TypedValueParser;
use ::clap::Arg;
use ::clap::Command;
use ::clap::Error;

use crate::Variants;

/// Import this trait to call [`value_parser`](ArgType::value_parser) on a
/// cognomen enum.
///
/// Blanket-implemented for every [`Variants`] type that is [`Clone`] and
/// [`FromStr`] (the derive emits both). The defining crate can stay
/// `no_std`; only the clap binary enables this feature.
pub trait ArgType: Variants + FromStr + Clone + Send + Sync + 'static {
    /// Clap parser that accepts any [`crate::FromLabel`] string and lists
    /// [`Variants::LABELS`] in `--help`.
    fn value_parser() -> Parser<Self> {
        Parser::new()
    }
}

impl<T> ArgType for T where T: Variants + FromStr + Clone + Send + Sync + 'static {}

/// Clap [`TypedValueParser`] for a cognomen enum.
///
/// Built by [`ArgType::value_parser`]. Parse uses [`FromStr`] (every
/// declared case, `rename`, and `alias`; unmatched input becomes an
/// `unknown` variant if marked). Help lists the default [`Variants::LABELS`].
#[derive(Clone, Copy, Debug)]
pub struct Parser<T> {
    _ty: PhantomData<fn() -> T>,
}

impl<T> Parser<T> {
    /// Empty parser; `T` is carried only in the type.
    #[must_use]
    pub const fn new() -> Self {
        Self { _ty: PhantomData }
    }
}

impl<T> Default for Parser<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TypedValueParser for Parser<T>
where
    T: Variants + FromStr + Clone + Send + Sync + 'static,
    T::Err: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<T, Error> {
        StringValueParser::new()
            .try_map(|s: std::string::String| T::from_str(&s))
            .parse_ref(cmd, arg, value)
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(T::LABELS.iter().copied().map(PossibleValue::new)))
    }
}
