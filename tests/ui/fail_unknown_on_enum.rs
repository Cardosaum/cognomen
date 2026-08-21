use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, unknown)]
enum Mode {
    A,
}

fn main() {}
