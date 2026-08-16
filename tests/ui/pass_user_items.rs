use cognomen::{Cognomen, Label, Variants};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower)]
enum Kind {
    Mic,
    App,
}

impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("user")
    }
}

impl Kind {
    pub const VARIANTS: &'static [&'static str] = &["mic", "app"];
    pub const LABELS: &'static [&'static str] = &["user-mic", "user-app"];
}

fn main() {
    assert_eq!(Kind::VARIANTS, &["mic", "app"]);
    assert_eq!(Kind::LABELS, &["user-mic", "user-app"]);
    assert_eq!(<Kind as Variants>::VARIANTS, &[Kind::Mic, Kind::App]);
    assert_eq!(<Kind as Variants>::LABELS, &["mic", "app"]);
    assert_eq!(Kind::Mic.to_string(), "user");
    assert_eq!(Kind::Mic.as_str(), "mic");
}
