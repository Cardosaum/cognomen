use cognomen::{Case, Cognomen, FromLabel, Label, Variants};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum ProcessRole {
    #[cognomen(alias = "main")]
    Supervisor,
    Worker,
}

fn main() {
    assert_eq!(ProcessRole::Supervisor.label(), "supervisor");
    assert_eq!(ProcessRole::Supervisor.as_str(), "supervisor");
    assert_eq!(ProcessRole::Supervisor.in_case(Case::Kebab), "supervisor");
    assert_eq!(ProcessRole::LABELS, &["supervisor", "worker"]);
    assert_eq!(
        ProcessRole::from_label("main").unwrap(),
        ProcessRole::Supervisor
    );
    assert_eq!(
        ProcessRole::from_label("supervisor").unwrap(),
        ProcessRole::Supervisor
    );
    assert!(ProcessRole::from_label("").is_err());
    assert!(ProcessRole::Supervisor == "supervisor");
    assert!(ProcessRole::Supervisor != "main");
}
