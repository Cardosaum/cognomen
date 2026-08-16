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

    /// Ident converted to `case` (ignores `rename`).
    fn in_case(&self, case: Case) -> &'static str;
}

/// Parse a declared-case label or `rename` into `Self`.
///
/// Implemented only for fieldless enums: a label cannot rebuild a payload.
pub trait FromLabel: Sized {
    /// Parse any declared-case label, or a variant `rename`, into `Self`.
    ///
    /// Equivalent to [`TryFrom::try_from`](core::convert::TryFrom).
    fn from_label(s: &str) -> Result<Self, crate::FromLabelError>;
}
