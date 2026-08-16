use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    #[cognomen(blurb = "a", blurb = "b")]
    A,
}

fn main() {}
