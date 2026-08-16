use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower, no_display, no_variants)]
enum Kind {
    Mic,
    App,
}

fn main() {
    assert_eq!(Kind::Mic.as_str(), "mic");
    assert_eq!(Kind::App.label(), "app");
    assert_eq!(Kind::LABELS, &["mic", "app"]);
    assert_eq!(Kind::from_label("app").unwrap(), Kind::App);
}
