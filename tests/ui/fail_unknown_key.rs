use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, rename = "x")]
enum Mode {
    A,
}

fn main() {}
