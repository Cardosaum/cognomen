//! Runtime tests for `cognomen::clap::ArgType`.

use clap::builder::TypedValueParser as _;
use clap::Arg;
use clap::Command;
use clap::Parser;
use cognomen::clap::ArgType;
use cognomen::Cognomen;
use cognomen::Variants;
use rstest::rstest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Mode {
    SingleProcess,
    MultiProcess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower)]
enum Kind {
    Mic,
    System,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Wire {
    #[cognomen(rename = "io_error")]
    IoFailed,
    OpenFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Role {
    #[cognomen(alias = "main")]
    Supervisor,
    Worker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Channel {
    Trades,
    #[cognomen(unknown)]
    Other,
}

#[derive(Debug, Parser)]
#[command(name = "prog")]
struct ModeCli {
    #[arg(long, value_parser = Mode::value_parser())]
    mode: Mode,
}

#[derive(Debug, Parser)]
#[command(name = "prog")]
struct KindCli {
    #[arg(long, value_parser = Kind::value_parser())]
    kind: Kind,
}

#[derive(Debug, Parser)]
#[command(name = "prog")]
struct WireCli {
    #[arg(long, value_parser = Wire::value_parser())]
    wire: Wire,
}

#[derive(Debug, Parser)]
#[command(name = "prog")]
struct RoleCli {
    #[arg(long, value_parser = Role::value_parser())]
    role: Role,
}

#[derive(Debug, Parser)]
#[command(name = "prog")]
struct ChannelCli {
    #[arg(long, value_parser = Channel::value_parser())]
    channel: Channel,
}

#[derive(Debug, Parser)]
#[command(name = "prog")]
struct OptKindCli {
    #[arg(long, value_parser = Kind::value_parser())]
    kind: Option<Kind>,
}

fn possible<T: ArgType>() -> Vec<String>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    T::value_parser()
        .possible_values()
        .expect("labels")
        .map(|v| v.get_name().to_owned())
        .collect()
}

fn cmd(name: &'static str, parser: impl clap::builder::TypedValueParser) -> Command {
    Command::new("prog").arg(Arg::new(name).long(name).value_parser(parser))
}

#[rstest]
#[case::snake("single_process", Mode::SingleProcess)]
#[case::kebab("single-process", Mode::SingleProcess)]
#[case::multi_snake("multi_process", Mode::MultiProcess)]
#[case::multi_kebab("multi-process", Mode::MultiProcess)]
fn mode_parses_declared_labels(#[case] input: &str, #[case] want: Mode) {
    let cli = ModeCli::try_parse_from(["prog", "--mode", input]).unwrap();
    assert_eq!(cli.mode, want);
}

#[rstest]
#[case::mic("mic", Kind::Mic)]
#[case::system("system", Kind::System)]
#[case::app("app", Kind::App)]
fn kind_parses_lower_labels(#[case] input: &str, #[case] want: Kind) {
    let cli = KindCli::try_parse_from(["prog", "--kind", input]).unwrap();
    assert_eq!(cli.kind, want);
}

#[rstest]
#[case::rename("io_error", Wire::IoFailed)]
#[case::rename_snake("io_failed", Wire::IoFailed)]
#[case::rename_kebab("io-failed", Wire::IoFailed)]
#[case::open_snake("open_failed", Wire::OpenFailed)]
#[case::open_kebab("open-failed", Wire::OpenFailed)]
fn wire_parses_rename_and_aliases(#[case] input: &str, #[case] want: Wire) {
    let cli = WireCli::try_parse_from(["prog", "--wire", input]).unwrap();
    assert_eq!(cli.wire, want);
}

#[rstest]
#[case::label("supervisor", Role::Supervisor)]
#[case::alias("main", Role::Supervisor)]
#[case::worker("worker", Role::Worker)]
fn role_parses_alias(#[case] input: &str, #[case] want: Role) {
    let cli = RoleCli::try_parse_from(["prog", "--role", input]).unwrap();
    assert_eq!(cli.role, want);
}

#[test]
fn role_rejects_empty_alias() {
    assert!(RoleCli::try_parse_from(["prog", "--role", ""]).is_err());
}

#[rstest]
#[case::known("trades", Channel::Trades)]
#[case::unknown("nope", Channel::Other)]
#[case::empty("", Channel::Other)]
fn channel_unknown_accepts_unmatched(#[case] input: &str, #[case] want: Channel) {
    let cli = ChannelCli::try_parse_from(["prog", "--channel", input]).unwrap();
    assert_eq!(cli.channel, want);
}

#[rstest]
#[case::unknown("nope")]
#[case::pascal("SingleProcess")]
#[case::screaming("SINGLE_PROCESS")]
fn mode_rejects_unknown_labels(#[case] input: &str) {
    let err = ModeCli::try_parse_from(["prog", "--mode", input]).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains(input), "{msg}");
}

#[test]
fn mode_rejects_empty_label() {
    assert!(ModeCli::try_parse_from(["prog", "--mode", ""]).is_err());
}

#[rstest]
#[case::upper("MIC")]
#[case::snake("source_kind")]
fn kind_rejects_wrong_case(#[case] input: &str) {
    assert!(KindCli::try_parse_from(["prog", "--kind", input]).is_err());
}

#[rstest]
#[case::absent(&["prog"][..], None)]
#[case::present(&["prog", "--kind", "app"][..], Some(Kind::App))]
fn option_kind(#[case] args: &[&str], #[case] want: Option<Kind>) {
    let cli = OptKindCli::try_parse_from(args).unwrap();
    assert_eq!(cli.kind, want);
}

#[rstest]
#[case::ok("app", Some(Kind::App))]
#[case::alias_rejected("MIC", None)]
fn builder_parses_kind(#[case] input: &str, #[case] want: Option<Kind>) {
    let command = cmd("kind", Kind::value_parser());
    let got = command.try_get_matches_from(["prog", "--kind", input]);
    match want {
        Some(kind) => {
            let matches = got.expect("parse");
            assert_eq!(matches.get_one::<Kind>("kind").copied(), Some(kind));
        }
        None => assert!(got.is_err()),
    }
}

#[rstest]
#[case::mode(possible::<Mode>(), Mode::LABELS)]
#[case::kind(possible::<Kind>(), Kind::LABELS)]
#[case::wire(possible::<Wire>(), Wire::LABELS)]
#[case::role(possible::<Role>(), Role::LABELS)]
fn possible_values_are_default_labels(#[case] got: Vec<String>, #[case] want: &[&str]) {
    assert_eq!(got, want);
}

#[rstest]
#[case::mode(cmd("mode", Mode::value_parser()), Mode::LABELS, &["single-process"])]
#[case::kind(cmd("kind", Kind::value_parser()), Kind::LABELS, &["MIC"])]
#[case::wire(cmd("wire", Wire::value_parser()), Wire::LABELS, &["io-failed"])]
#[case::role(cmd("role", Role::value_parser()), Role::LABELS, &["main"])]
fn help_lists_default_labels_not_aliases(
    #[case] mut command: Command,
    #[case] labels: &[&str],
    #[case] aliases: &[&str],
) {
    let help = command.render_long_help().to_string();
    for label in labels {
        assert!(help.contains(label), "missing {label} in {help}");
    }
    for alias in aliases {
        assert!(!help.contains(alias), "alias {alias} in {help}");
    }
}

fn assert_copy<T: Copy>(value: T) -> (T, T) {
    (value, value)
}

#[test]
fn parser_is_copy_and_default() {
    let (a, b) = assert_copy(Mode::value_parser());
    let _ = (a, b, cognomen::clap::Parser::<Mode>::default());
}
