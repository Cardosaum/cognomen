use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(lower)]
enum Collide {
    Zero,
    zero, // same lowercase label as `Zero`, so the reverse match would conflict
}

fn main() {}