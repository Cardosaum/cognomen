use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, prefix = "")]
enum Mode {
    A,
}

fn main() {}
