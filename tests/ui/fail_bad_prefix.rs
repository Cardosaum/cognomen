use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, prefix = "my-label")]
enum Mode {
    A,
}

fn main() {}
