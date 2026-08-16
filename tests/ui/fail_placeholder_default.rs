use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, reason = "host open failed {cause}")]
enum Mode {
    OpenFailed { cause: String },
}

fn main() {}
