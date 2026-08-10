use cognomen::Labeled;

#[derive(Labeled)]
#[labeled(kebab-case)]
enum Mode {
    SingleProcess,
    MultiProcess,
}

fn main() {
    assert_eq!(Mode::SingleProcess.as_str(), "single-process");
    assert_eq!(Mode::MultiProcess.label(), "multi-process");
}
