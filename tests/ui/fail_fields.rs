use cognomen::Labeled;

#[derive(Labeled)]
#[labeled(snake_case)]
enum Mode {
    Unit,
    WithField(u8),
}

fn main() {}
