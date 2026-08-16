use cognomen::Cognomen;

#[derive(Cognomen)]
#[cognomen(snake_case, prefix = "cfg", cfg_snake = "")]
enum Mode {
    A,
}

fn main() {}
