use cognomen::{Case, Cognomen, FromLabel, Label, Variants};

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
    assert_eq!(Transport::UnixSocket.label(), "unix_socket");
    assert_eq!(Transport::WebSocket.in_case(Case::Kebab), "web-socket");
    assert_eq!(Transport::UnixSocket.in_case(Case::Kebab), "unix-socket");
    assert_eq!(Transport::WebSocket.in_case(Case::Snake), "web_socket");

    // Default is chosen by order: PascalCase here, not snake_case.
    assert_eq!(Direction::LeftHand.label(), "LeftHand");
    assert_eq!(Direction::RightHand.label(), "RightHand");
    assert_eq!(Direction::LeftHand.in_case(Case::Snake), "left_hand");

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
    assert_eq!(err.input, "nope");

    // Building-block aliases.
    assert_eq!(Transport::WebSocket.as_str(), "web_socket");
    assert_eq!(Transport::WebSocket.as_str(), "web_socket");
    assert_eq!(
        core::convert::AsRef::<str>::as_ref(&Transport::UnixSocket),
        "unix_socket"
    );
    assert_eq!(
        Transport::VARIANTS,
        &[Transport::WebSocket, Transport::UnixSocket]
    );
    assert_eq!(Transport::LABELS, &["web_socket", "unix_socket"]);
    assert!(Transport::WebSocket == "web-socket");
    assert!("web_socket" == Transport::WebSocket);
    assert_eq!(
        Transport::from_label("web-socket").unwrap(),
        Transport::WebSocket
    );
}