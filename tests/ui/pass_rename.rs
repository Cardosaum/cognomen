use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Wire {
    #[cognomen(rename = "io_error")]
    IoFailed,
    OpenFailed,
}

fn main() {
    assert_eq!(Wire::IoFailed.label(), "io_error");
    assert_eq!(Wire::IoFailed.as_str(), "io_error");
    assert_eq!(Wire::IoFailed.label_snake(), "io_failed");
    assert_eq!(Wire::IoFailed.label_kebab(), "io-failed");
    assert_eq!(Wire::OpenFailed.label(), "open_failed");

    assert_eq!(Wire::from_label("io_error").unwrap(), Wire::IoFailed);
    assert_eq!(Wire::try_from("io_failed"), Ok(Wire::IoFailed));
    assert_eq!(Wire::try_from("io-failed"), Ok(Wire::IoFailed));
    assert!(Wire::IoFailed == "io_error");
}
