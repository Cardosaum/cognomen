use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    #[cognomen(rename = "a", rename = "b")]
    A,
}

fn main() {}
