use cognomen::{Cognomen, Variants};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    SingleProcess,
    MultiProcess,
}

const fn label_in_const() -> &'static str {
    Mode::SingleProcess.label()
}

fn main() {
    assert_eq!(label_in_const(), "single_process");
    const TABLES: &[&str] = Mode::LABELS;
    assert_eq!(TABLES, &["single_process", "multi_process"]);
}
