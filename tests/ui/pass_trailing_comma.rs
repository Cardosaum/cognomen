use cognomen::{Case, Cognomen, FromLabel, Label};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case,)]
enum Kind {
    #[cognomen(rename = "mic",)]
    Mic,
    App,
}

fn main() {
    assert_eq!(Kind::Mic.as_str(), "mic");
    assert_eq!(Kind::App.in_case(Case::Kebab), "app");
    assert_eq!(Kind::from_label("mic").unwrap(), Kind::Mic);
}
