use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(camelCase)]
enum Camel {
    SingleProcess,
}

#[derive(Cognomen)]
#[cognomen(PascalCase)]
enum Pascal {
    SingleProcess,
}

#[derive(Cognomen)]
#[cognomen(SCREAMING_SNAKE_CASE)]
enum Screaming {
    SingleProcess,
}

fn main() {
    assert_eq!(Camel::SingleProcess.label(), "singleProcess");
    assert_eq!(Pascal::SingleProcess.label(), "SingleProcess");
    assert_eq!(Screaming::SingleProcess.label(), "SINGLE_PROCESS");
}
