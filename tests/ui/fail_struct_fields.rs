use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    Unit,
    Named { x: u8 },
}

fn main() {}
