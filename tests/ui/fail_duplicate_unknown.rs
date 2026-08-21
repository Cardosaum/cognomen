use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    #[cognomen(unknown)]
    A,
    #[cognomen(unknown)]
    B,
}

fn main() {}
