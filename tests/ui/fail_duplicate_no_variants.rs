use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(lower, no_variants, no_variants)]
enum Mode {
    A,
}

fn main() {}
