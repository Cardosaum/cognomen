use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
#[cognomen(kebab-case)]
enum Mode {
    A,
}

fn main() {}
