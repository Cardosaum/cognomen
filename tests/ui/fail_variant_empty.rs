use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    #[cognomen()]
    A,
}

fn main() {}
