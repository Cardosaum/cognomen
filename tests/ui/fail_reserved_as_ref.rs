use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, as_ref = "x")]
enum Mode {
    A,
}

fn main() {}
