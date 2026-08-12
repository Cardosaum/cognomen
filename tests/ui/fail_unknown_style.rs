use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(not_a_case)]
enum Mode {
    A,
}

fn main() {}
