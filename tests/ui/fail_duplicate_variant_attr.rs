use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    #[cognomen(blurb = "a")]
    #[cognomen(blurb = "a")]
    A,
}

fn main() {}
