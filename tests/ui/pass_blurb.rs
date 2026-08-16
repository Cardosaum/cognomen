use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(lower, blurb = "")]
enum SourceKind {
    #[cognomen(blurb = "microphone / input device")]
    Mic,
    #[cognomen(blurb = "system-wide loopback")]
    System,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, blurb = "n/a", hint = "use default")]
enum Wire {
    #[cognomen(blurb = "input failed", hint = "check the device")]
    IoFailed,
    OpenFailed,
}

// A key that appears only on variants still becomes a method; others use as_str.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, prefix = "cfg")]
enum Feature {
    #[cognomen(help = "write logs to stderr")]
    EnableLogging,
    EnableTracing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Renamed {
    #[cognomen(rename = "io_error", blurb = "input failed")]
    IoFailed,
    #[cognomen(rename = "open_error")]
    OpenFailed,
}

fn main() {
    assert_eq!(SourceKind::Mic.as_str(), "mic");
    assert_eq!(SourceKind::Mic.blurb(), "microphone / input device");
    assert_eq!(SourceKind::System.blurb(), "system-wide loopback");
    assert_eq!(SourceKind::App.blurb(), "");

    assert_eq!(Wire::IoFailed.blurb(), "input failed");
    assert_eq!(Wire::OpenFailed.blurb(), "n/a");
    assert_eq!(Wire::IoFailed.hint(), "check the device");
    assert_eq!(Wire::OpenFailed.hint(), "use default");

    assert_eq!(Feature::EnableLogging.cfg_snake(), "enable_logging");
    assert_eq!(Feature::EnableLogging.help(), "write logs to stderr");
    assert_eq!(Feature::EnableTracing.help(), "enable_tracing");

    assert_eq!(Renamed::IoFailed.as_str(), "io_error");
    assert_eq!(Renamed::IoFailed.blurb(), "input failed");
    assert_eq!(Renamed::OpenFailed.blurb(), "open_error");

    assert_eq!(SourceKind::from_label("mic").unwrap(), SourceKind::Mic);
    assert!(SourceKind::Mic == "mic");
    assert!(SourceKind::try_from("microphone / input device").is_err());
}
