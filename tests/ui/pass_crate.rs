use cognomen::{Cognomen, FromLabel, Label, Variants};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower, crate = ::cognomen)]
enum Kind {
    Mic,
    App,
}

fn main() {
    assert_eq!(Kind::Mic.as_str(), "mic");
    assert_eq!(Kind::from_label("app").unwrap(), Kind::App);
    assert_eq!(Kind::LABELS, &["mic", "app"]);
}
