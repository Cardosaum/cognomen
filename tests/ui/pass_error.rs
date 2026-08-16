use cognomen::{Cognomen, Label};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower)]
enum Status {
    Error,
    Err,
    Ok,
}

fn main() {
    assert_eq!(Status::Error.as_str(), "error");
    assert_eq!(Status::try_from("err"), Ok(Status::Err));
    assert_eq!("ok".parse::<Status>().unwrap(), Status::Ok);
}
