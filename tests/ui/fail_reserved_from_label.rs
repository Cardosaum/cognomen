use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, from_label = "x")]
enum Mode {
    A,
}

fn main() {}
