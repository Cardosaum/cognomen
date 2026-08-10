use cognomen::Cognomen;

// Default = first listed (snake_case); kebab-case is the alternate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Transport {
    WebSocket,
    UnixSocket,
}

// Default = PascalCase (first listed); snake_case is the alternate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(PascalCase, snake_case)]
enum Direction {
    LeftHand,
    RightHand,
}

fn main() {
    assert_eq!(Transport::WebSocket.label(), "web_socket");
    assert_eq!(Transport::UnixSocket.as_str(), "unix_socket");
    assert_eq!(Transport::WebSocket.label_kebab(), "web-socket");
    assert_eq!(Transport::UnixSocket.label_kebab(), "unix-socket");

    // Every declared case is addressable, including the default.
    assert_eq!(Transport::WebSocket.label_snake(), "web_socket");

    // Default is chosen by order: PascalCase here, not snake_case.
    assert_eq!(Direction::LeftHand.label(), "LeftHand");
    assert_eq!(Direction::RightHand.as_str(), "RightHand");
    assert_eq!(Direction::LeftHand.label_snake(), "left_hand");

    // Reverse path: parse any declared case back to the variant.
    assert_eq!(Transport::try_from("web_socket"), Ok(Transport::WebSocket));
    assert_eq!(Transport::try_from("web-socket"), Ok(Transport::WebSocket));
    assert_eq!("unix_socket".parse::<Transport>(), Ok(Transport::UnixSocket));
    assert_eq!("unix-socket".parse::<Transport>(), Ok(Transport::UnixSocket));
    assert!(Transport::try_from("hovercar").is_err());

    // Every declared case parses, including the non-default one.
    assert_eq!(Direction::try_from("left_hand"), Ok(Direction::LeftHand));
    assert_eq!(Direction::try_from("LeftHand"), Ok(Direction::LeftHand));
    assert!("carrier".parse::<Direction>().is_err());

    // The error reports the offending input and satisfies std::error::Error.
    let err = Transport::try_from("nope").unwrap_err();
    assert!(err.to_string().contains("nope"));
    let _: &dyn std::error::Error = &err;
}