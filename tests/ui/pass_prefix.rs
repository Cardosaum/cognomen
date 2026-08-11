use cognomen::Cognomen;

// Custom prefix changes the accessor method names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case, prefix = "my_label")]
enum Transport {
    WebSocket,
    UnixSocket,
}

// Prefix works with a single case style too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, prefix = "cfg")]
enum Feature {
    EnableLogging,
    EnableTracing,
}

fn main() {
    // Label values are unchanged — prefix only affects accessor names.
    assert_eq!(Transport::WebSocket.label(), "web_socket");
    assert_eq!(Transport::UnixSocket.as_str(), "unix_socket");

    // Per-case accessors use the custom prefix.
    assert_eq!(Transport::WebSocket.my_label_snake(), "web_socket");
    assert_eq!(Transport::UnixSocket.my_label_kebab(), "unix-socket");

    // Reverse path works with the same label values.
    assert_eq!(Transport::try_from("web_socket"), Ok(Transport::WebSocket));
    assert_eq!("web_socket".parse::<Transport>(), Ok(Transport::WebSocket));

    assert!(Transport::try_from("nope").is_err());

    // Single-case prefix works.
    assert_eq!(Feature::EnableLogging.label(), "enable_logging");
    assert_eq!(Feature::EnableTracing.cfg_snake(), "enable_tracing");

    // Reverse path for single-case prefix.
    assert_eq!(Feature::try_from("enable_logging"), Ok(Feature::EnableLogging));
    assert!(Feature::try_from("nope").is_err());
}
