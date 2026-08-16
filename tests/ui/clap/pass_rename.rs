use cognomen::clap::ArgType;
use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Wire {
    #[cognomen(rename = "io_error")]
    IoFailed,
    OpenFailed,
}

fn main() {
    let _ = Wire::value_parser();
}
