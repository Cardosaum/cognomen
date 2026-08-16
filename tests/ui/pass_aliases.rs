use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake, kebab, camel_case, pascal, screaming, lowercase, uppercase, title_case)]
enum Mode {
    SingleProcess,
}

fn main() {
    assert_eq!(Mode::SingleProcess.label(), "single_process");
    assert_eq!(Mode::SingleProcess.label_kebab(), "single-process");
    assert_eq!(Mode::SingleProcess.label_camel(), "singleProcess");
    assert_eq!(Mode::SingleProcess.label_pascal(), "SingleProcess");
    assert_eq!(Mode::SingleProcess.label_screaming_snake(), "SINGLE_PROCESS");
    assert_eq!(Mode::SingleProcess.label_lower(), "singleprocess");
    assert_eq!(Mode::SingleProcess.label_upper(), "SINGLEPROCESS");
    assert_eq!(Mode::SingleProcess.label_title(), "Single Process");
}
