use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, kebab-case, snake_case)]
enum Duplicate {
    A,
}

fn main() {}