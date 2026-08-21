//! Label accessors live on [`Label`], not as inherent methods on the enum.

/// Case style used by [`Label::in_case`].
///
/// The derive converts from the variant ident (not `rename`). `label()` is
/// the default case from `#[cognomen(...)]`, or the variant `rename`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Case {
    /// `variant_name`
    Snake,
    /// `variant-name`
    Kebab,
    /// `variantName`
    Camel,
    /// `VariantName`
    Pascal,
    /// `VARIANT_NAME`
    ScreamingSnake,
    /// `variantname`
    Lower,
    /// `VARIANTNAME`
    Upper,
    /// `Variant Name`
    Title,
}

/// Stable string label for a cognomen enum.
///
/// Import this trait to call `e.label()` / `e.as_str()`. A user inherent
/// `fn as_str` still compiles; use `<E as Label>::as_str(&e)`.
///
/// [`PartialEq<str>`](core::cmp::PartialEq) on the enum (generated) compares
/// this label, not an extra such as `reason`.
pub trait Label {
    /// Default-case label, or the variant `rename`.
    fn label(&self) -> &'static str;

    /// Alias of [`label`](Self::label).
    fn as_str(&self) -> &'static str {
        self.label()
    }

    /// Ident converted to `case` (ignores `rename` and `alias`).
    fn in_case(&self, case: Case) -> &'static str;
}

/// Parse a declared-case label, `rename`, or `alias` into `Self`.
///
/// Implemented only for fieldless enums: a label cannot rebuild a payload.
/// A variant marked `#[cognomen(unknown)]` receives unmatched strings
/// instead of [`crate::FromLabelError`]; that path is not a bijection.
/// clap `value_parser` does not follow that fallback.
pub trait FromLabel: Sized {
    /// Parse any declared-case label, a variant `rename`, or a variant
    /// `alias`, into `Self`.
    ///
    /// Equivalent to [`TryFrom::try_from`](core::convert::TryFrom). When the
    /// enum marks a unit variant `unknown`, unmatched input becomes that
    /// variant and this never returns [`crate::FromLabelError`].
    fn from_label(s: &str) -> Result<Self, crate::FromLabelError>;
}

/// Closed parse used by clap: declared cases, `rename`, and `alias` only.
///
/// Does not follow `#[cognomen(unknown)]`. Unmatched input is always an
/// error so flag garbage cannot become the fallback variant.
#[doc(hidden)]
pub trait __FromDeclared: Sized {
    /// Parse a declared-case label, `rename`, or `alias`, never the unknown
    /// fallback.
    fn __from_declared(s: &str) -> Result<Self, crate::FromLabelError>;
}
