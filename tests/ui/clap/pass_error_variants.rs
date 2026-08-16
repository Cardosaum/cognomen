use cognomen::clap::ArgType;
use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower)]
enum Status {
    Error,
    Err,
    Ok,
}

fn main() {
    let _ = Status::value_parser();
}
