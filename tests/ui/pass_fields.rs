use cognomen::{Cognomen, Label};

#[derive(Debug, Clone, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    Unit,
    WithField(u8),
    Named { x: u8 },
}

fn main() {
    assert_eq!(Mode::Unit.as_str(), "unit");
    assert_eq!(Mode::WithField(1).as_str(), "with_field");
    assert_eq!(Mode::Named { x: 2 }.label(), "named");
    assert!(Mode::WithField(1) == "with_field");
}
