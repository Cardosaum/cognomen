use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, alias = "main")]
enum Mode {
    A,
}

fn main() {}
