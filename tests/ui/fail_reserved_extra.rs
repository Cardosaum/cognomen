use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, label = "")]
enum Mode {
    A,
}

fn main() {}
