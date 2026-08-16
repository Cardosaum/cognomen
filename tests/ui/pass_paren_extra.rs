use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower, blurb())]
enum Kind {
    #[cognomen(blurb = "mic")]
    Mic,
    App,
}

fn main() {
    assert_eq!(Kind::Mic.blurb(), "mic");
    assert_eq!(Kind::App.blurb(), "");
}
