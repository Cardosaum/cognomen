use cognomen::clap::ArgType;
use cognomen::Cognomen;

#[derive(Debug, Cognomen)]
#[cognomen(lower)]
enum Kind {
    Mic,
}

fn needs_parser<T: ArgType>() {}

fn main() {
    needs_parser::<Kind>();
}
