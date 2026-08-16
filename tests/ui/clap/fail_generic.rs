use cognomen::clap::ArgType;
use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Flag<const N: usize> {
    On,
    Off,
}

fn needs_parser<T: ArgType>() {}

fn main() {
    needs_parser::<Flag<1>>();
}
