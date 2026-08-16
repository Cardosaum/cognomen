use cognomen::{Cognomen, Label, Reason};

#[derive(Debug, Clone, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum HostError {
    #[cognomen(reason = "host backend unsupported {capability}")]
    Unsupported { capability: &'static str },
    #[cognomen(reason = "host open failed {cause}")]
    OpenFailed { cause: String },
    #[cognomen(reason = "host refused request {why}")]
    BadRequest { why: &'static str },
    #[cognomen(reason = "host io failed {status}")]
    Io { status: String },
}

fn main() {
    let e = HostError::OpenFailed {
        cause: String::from("busy"),
    };
    assert_eq!(e.as_str(), "open_failed");
    assert_eq!(e.reason(), "host open failed busy");
    assert_eq!(
        HostError::Unsupported {
            capability: "x"
        }
        .reason(),
        "host backend unsupported x"
    );
    assert_eq!(
        format!(
            "{}",
            HostError::BadRequest {
                why: "device name has interior NUL"
            }
            .reason()
        ),
        "host refused request device name has interior NUL"
    );
}
