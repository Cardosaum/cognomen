use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(lower)]
enum Collide {
    Zero,
    #[cognomen(alias = "zero")]
    Other,
}

fn main() {}
