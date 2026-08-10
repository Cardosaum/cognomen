use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    Unit,
    WithField(u8),
}

fn main() {}
