use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, rename())]
enum Mode {
    A,
}

fn main() {}
