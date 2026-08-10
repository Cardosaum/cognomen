use cognomen::Labeled;

#[derive(Labeled)]
#[labeled(snake_case)]
struct NotAnEnum {
    x: u8,
}

fn main() {}
