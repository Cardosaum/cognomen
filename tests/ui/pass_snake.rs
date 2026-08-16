use cognomen::{Cognomen, Label};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Mode {
    SingleProcess,
    MultiProcess,
    Hybrid,
}

fn main() {
    assert_eq!(Mode::SingleProcess.label(), "single_process");
    assert_eq!(Mode::MultiProcess.label(), "multi_process");
    assert_eq!(Mode::Hybrid.label(), "hybrid");
}
