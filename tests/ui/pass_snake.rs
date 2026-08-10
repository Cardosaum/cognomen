use cognomen::Labeled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Labeled)]
#[labeled(snake_case)]
enum Mode {
    SingleProcess,
    MultiProcess,
    Hybrid,
}

fn main() {
    assert_eq!(Mode::SingleProcess.as_str(), "single_process");
    assert_eq!(Mode::MultiProcess.label(), "multi_process");
    assert_eq!(Mode::Hybrid.as_str(), "hybrid");
}
