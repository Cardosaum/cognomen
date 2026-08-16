use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(lower, no_display, no_display)]
enum Mode {
    A,
}

fn main() {}
