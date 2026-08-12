use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, prefix = "a", prefix = "b")]
enum Mode {
    A,
}

fn main() {}
