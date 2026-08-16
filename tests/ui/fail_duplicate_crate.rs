use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, crate = ::a, crate = ::b)]
enum Mode {
    A,
}

fn main() {}
