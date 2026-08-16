use cognomen::clap::ArgType;
use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower)]
enum Kind {
    Mic,
    App,
}

fn main() {
    let _ = Kind::value_parser();
    let _ = cognomen::clap::Parser::<Kind>::new();
    let _ = cognomen::clap::Parser::<Kind>::default();
}
