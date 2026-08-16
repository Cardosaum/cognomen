use cognomen::{Case, Cognomen, Label};

#[derive(Cognomen)]
#[cognomen(snake, kebab, camel_case, pascal, screaming, lowercase, uppercase, title_case)]
enum Mode {
    SingleProcess,
}

fn main() {
    assert_eq!(Mode::SingleProcess.label(), "single_process");
    assert_eq!(Mode::SingleProcess.in_case(Case::Kebab), "single-process");
    assert_eq!(Mode::SingleProcess.in_case(Case::Camel), "singleProcess");
    assert_eq!(Mode::SingleProcess.in_case(Case::Pascal), "SingleProcess");
    assert_eq!(
        Mode::SingleProcess.in_case(Case::ScreamingSnake),
        "SINGLE_PROCESS"
    );
    assert_eq!(Mode::SingleProcess.in_case(Case::Lower), "singleprocess");
    assert_eq!(Mode::SingleProcess.in_case(Case::Upper), "SINGLEPROCESS");
    assert_eq!(Mode::SingleProcess.in_case(Case::Title), "Single Process");
}
