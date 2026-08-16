use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, blurb = "", blurb = "")]
enum Mode {
    A,
}

fn main() {}
