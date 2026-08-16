use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, blurb(1))]
enum Mode {
    A,
}

fn main() {}
