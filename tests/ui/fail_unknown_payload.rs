use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    #[cognomen(unknown)]
    Other { tag: &'static str },
}

fn main() {}
