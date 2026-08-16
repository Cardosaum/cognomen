//! Extra strings live on traits, not as inherent methods on the enum.
//!
//! Every extra returns [`Formatted`](crate::Formatted): static text is one
//! literal piece; `{field}` interpolation appends args. Adding a placeholder
//! does not change the signature.

use crate::Formatted;

/// Tag-typed extra. Known keys (`ReasonKey`, ...) also get a named trait
/// (`Reason`) with a method of the same name.
pub trait Extra<K: ?Sized> {
    /// Extra text for this key, always [`Formatted`].
    fn extra(&self) -> Formatted<'_>;
}

/// Key type for [`Reason`].
pub enum ReasonKey {}

/// Extra `reason`. Import to call `e.reason()`. A user inherent `fn reason`
/// still compiles; use UFCS.
pub trait Reason {
    /// Extra `reason`, always [`Formatted`].
    fn reason(&self) -> Formatted<'_>;
}

impl<T: Extra<ReasonKey> + ?Sized> Reason for T {
    #[inline]
    fn reason(&self) -> Formatted<'_> {
        Extra::<ReasonKey>::extra(self)
    }
}

/// Key type for [`Blurb`].
pub enum BlurbKey {}

/// Extra `blurb`. Import to call `e.blurb()`. A user inherent `fn blurb`
/// still compiles; use UFCS.
pub trait Blurb {
    /// Extra `blurb`, always [`Formatted`].
    fn blurb(&self) -> Formatted<'_>;
}

impl<T: Extra<BlurbKey> + ?Sized> Blurb for T {
    #[inline]
    fn blurb(&self) -> Formatted<'_> {
        Extra::<BlurbKey>::extra(self)
    }
}

/// Key type for [`Hint`].
pub enum HintKey {}

/// Extra `hint`. Import to call `e.hint()`. A user inherent `fn hint`
/// still compiles; use UFCS.
pub trait Hint {
    /// Extra `hint`, always [`Formatted`].
    fn hint(&self) -> Formatted<'_>;
}

impl<T: Extra<HintKey> + ?Sized> Hint for T {
    #[inline]
    fn hint(&self) -> Formatted<'_> {
        Extra::<HintKey>::extra(self)
    }
}

/// Key type for [`Help`].
pub enum HelpKey {}

/// Extra `help`. Import to call `e.help()`. A user inherent `fn help`
/// still compiles; use UFCS.
pub trait Help {
    /// Extra `help`, always [`Formatted`].
    fn help(&self) -> Formatted<'_>;
}

impl<T: Extra<HelpKey> + ?Sized> Help for T {
    #[inline]
    fn help(&self) -> Formatted<'_> {
        Extra::<HelpKey>::extra(self)
    }
}
