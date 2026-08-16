use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    #[cognomen(reason = "host open failed {nope}")]
    OpenFailed { cause: String },
}

fn main() {}
