use cognomen::Labeled;

#[derive(Labeled)]
#[labeled(camelCase)]
enum Camel {
    SingleProcess,
}

#[derive(Labeled)]
#[labeled(PascalCase)]
enum Pascal {
    SingleProcess,
}

#[derive(Labeled)]
#[labeled(SCREAMING_SNAKE_CASE)]
enum Screaming {
    SingleProcess,
}

fn main() {
    assert_eq!(Camel::SingleProcess.as_str(), "singleProcess");
    assert_eq!(Pascal::SingleProcess.as_str(), "SingleProcess");
    assert_eq!(Screaming::SingleProcess.as_str(), "SINGLE_PROCESS");
}
