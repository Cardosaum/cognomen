use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    Named { x: u8 },
    #[cognomen(unknown)]
    Other,
}

fn main() {}
