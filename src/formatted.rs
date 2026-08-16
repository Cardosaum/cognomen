//! Extra strings (`Reason::reason`, ...). Always this type, never `&'static str`.

use core::fmt::{self, Write};

const MAX_PIECES: usize = 16;

#[derive(Clone, Copy)]
enum Piece<'a> {
    Lit(&'static str),
    Arg(&'a dyn fmt::Display),
}

/// Extra text: one or more literal pieces plus borrowed field args.
///
/// Every extra returns this type. Static text is a single literal; `{field}`
/// interpolation appends args. Adding a placeholder does not change the
/// method signature.
///
/// Comparison writes the same text as [`Display`] and does not allocate:
/// `e.reason() == "host open failed busy"`.
///
/// ```
/// use cognomen::{Cognomen, Reason};
///
/// #[derive(Cognomen)]
/// #[cognomen(snake_case)]
/// enum HostError {
///     #[cognomen(reason = "host open failed {cause}")]
///     OpenFailed { cause: &'static str },
/// }
///
/// let e = HostError::OpenFailed { cause: "busy" };
/// assert_eq!(e.reason(), "host open failed busy");
/// ```
///
/// Print with `write!(f, "{}", e.reason())`. With `alloc`,
/// `e.reason().to_string()` also works.
#[derive(Clone, Copy)]
pub struct Formatted<'a> {
    pieces: [Option<Piece<'a>>; MAX_PIECES],
    n: u8,
}

impl<'a> Formatted<'a> {
    /// Empty interpolation (no pieces yet). Used by the derive.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            pieces: [None; MAX_PIECES],
            n: 0,
        }
    }

    /// Append a literal fragment. Used by the derive.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn lit(self, s: &'static str) -> Self {
        self.push(Piece::Lit(s))
    }

    /// Append a payload field. Used by the derive.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub fn arg(self, value: &'a dyn fmt::Display) -> Self {
        self.push(Piece::Arg(value))
    }

    fn push(mut self, piece: Piece<'a>) -> Self {
        let i = self.n as usize;
        debug_assert!(
            i < MAX_PIECES,
            "cognomen extra exceeded {MAX_PIECES} pieces"
        );
        self.pieces[i] = Some(piece);
        self.n += 1;
        self
    }
}

impl fmt::Display for Formatted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for piece in self.pieces.iter().take(self.n as usize) {
            match piece {
                Some(Piece::Lit(s)) => f.write_str(s)?,
                Some(Piece::Arg(v)) => write!(f, "{v}")?,
                None => {}
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Formatted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl PartialEq<str> for Formatted<'_> {
    fn eq(&self, other: &str) -> bool {
        let mut cmp = Cmp {
            rest: other,
            ok: true,
        };
        let _ = write!(&mut cmp, "{self}");
        cmp.ok && cmp.rest.is_empty()
    }
}

impl PartialEq<&str> for Formatted<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<Formatted<'_>> for str {
    #[inline]
    fn eq(&self, other: &Formatted<'_>) -> bool {
        other == self
    }
}

impl PartialEq<Formatted<'_>> for &str {
    #[inline]
    fn eq(&self, other: &Formatted<'_>) -> bool {
        other == *self
    }
}

struct Cmp<'a> {
    rest: &'a str,
    ok: bool,
}

impl Write for Cmp<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !self.ok {
            return Ok(());
        }
        match self.rest.strip_prefix(s) {
            Some(tail) => self.rest = tail,
            None => self.ok = false,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Formatted;

    #[test]
    fn eq_and_display() {
        let f = Formatted::empty().lit("host open failed ").arg(&"busy");
        assert!(f == "host open failed busy");
        assert!(f == *"host open failed busy");
        assert!("host open failed busy" == f);
        assert!(f != "host open failed");
        assert!(f != "host open failed busy!");
        assert!(f != "Host open failed busy");
        assert!(Formatted::empty() == "");
        assert!(Formatted::empty().arg(&"x") == "x");
        assert!(Formatted::empty().lit("static") == "static");
    }
}
