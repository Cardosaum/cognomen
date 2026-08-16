use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, VARIANTS = "x")]
enum Mode {
    A,
}

fn main() {}
