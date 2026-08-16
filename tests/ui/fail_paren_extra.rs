use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, blurb("a", "b"))]
enum Mode {
    A,
}

fn main() {}
