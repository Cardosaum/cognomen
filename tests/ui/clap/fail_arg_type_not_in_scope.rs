use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower)]
enum Kind {
    Mic,
}

fn main() {
    let _ = Kind::value_parser();
}
