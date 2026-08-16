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

macro_rules! define_extra {
    ($trait:ident, $key:ident, $method:ident) => {
        #[doc = concat!("Key type for [`", stringify!($trait), "`].")]
        pub enum $key {}

        #[doc = concat!(
            "Extra `", stringify!($method), "`. Import to call `e.",
            stringify!($method), "()`. A user inherent `fn ",
            stringify!($method), "` still compiles; use UFCS."
        )]
        pub trait $trait {
            #[doc = concat!("Extra `", stringify!($method), "`, always [`Formatted`].")]
            fn $method(&self) -> Formatted<'_>;
        }

        impl<T: Extra<$key> + ?Sized> $trait for T {
            #[inline]
            fn $method(&self) -> Formatted<'_> {
                Extra::<$key>::extra(self)
            }
        }
    };
}

define_extra!(Reason, ReasonKey, reason);
define_extra!(Blurb, BlurbKey, blurb);
define_extra!(Hint, HintKey, hint);
define_extra!(Help, HelpKey, help);
