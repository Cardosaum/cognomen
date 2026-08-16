use cognomen::{Case, Cognomen, Label};

// prefix is accepted; case accessors live on Label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case, prefix = "my_label")]
enum Transport {
    WebSocket,
    UnixSocket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, prefix = "cfg")]
enum Feature {
    EnableLogging,
    EnableTracing,
}

fn main() {
    assert_eq!(Transport::WebSocket.label(), "web_socket");
    assert_eq!(Transport::UnixSocket.label(), "unix_socket");
    assert_eq!(Transport::WebSocket.in_case(Case::Snake), "web_socket");
    assert_eq!(Transport::UnixSocket.in_case(Case::Kebab), "unix-socket");

    assert_eq!(Transport::try_from("web_socket"), Ok(Transport::WebSocket));
    assert_eq!("web_socket".parse::<Transport>(), Ok(Transport::WebSocket));
    assert!(Transport::try_from("nope").is_err());

    assert_eq!(Feature::EnableLogging.label(), "enable_logging");
    assert_eq!(Feature::EnableTracing.in_case(Case::Snake), "enable_tracing");

    assert_eq!(Feature::try_from("enable_logging"), Ok(Feature::EnableLogging));
    assert!(Feature::try_from("nope").is_err());
}
