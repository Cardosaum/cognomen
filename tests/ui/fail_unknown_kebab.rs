use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(kebab-foo)]
enum Mode {
    A,
}

fn main() {}
