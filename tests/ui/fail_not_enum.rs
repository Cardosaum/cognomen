use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
struct NotAnEnum {
    x: u8,
}

fn main() {}
