use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(lower)]
enum Collide {
    Zero,
    #[cognomen(rename = "zero")]
    Other,
}

fn main() {}
