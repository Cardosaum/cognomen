use cognomen::{Cognomen, Label};

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

#[derive(Cognomen)]
#[cognomen(lower)]
enum Low {
    SingleProcess,
}

#[derive(Cognomen)]
#[cognomen(upper)]
enum Up {
    SingleProcess,
}

#[derive(Cognomen)]
#[cognomen(title)]
enum Title {
    SingleProcess,
}

fn main() {
    assert_eq!(Camel::SingleProcess.label(), "singleProcess");
    assert_eq!(Pascal::SingleProcess.label(), "SingleProcess");
    assert_eq!(Screaming::SingleProcess.label(), "SINGLE_PROCESS");

    assert_eq!(Low::SingleProcess.label(), "singleprocess");
    assert_eq!(Up::SingleProcess.label(), "SINGLEPROCESS");
    assert_eq!(Title::SingleProcess.label(), "Single Process");
}
