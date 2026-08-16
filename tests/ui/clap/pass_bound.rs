use cognomen::clap::ArgType;
use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower)]
enum Kind {
    Mic,
    App,
}

fn parser<T: ArgType>() -> cognomen::clap::Parser<T> {
    T::value_parser()
}

fn main() {
    let _ = parser::<Kind>();
}
