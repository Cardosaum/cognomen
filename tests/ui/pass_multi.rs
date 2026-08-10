use cognomen::Cognomen;

// Default = first listed (snake_case); kebab-case is the alternate.
#[derive(Cognomen)]
#[cognomen(snake_case, kebab-case)]
enum Transport {
    WebSocket,
    UnixSocket,
}

// Default = PascalCase (first listed); snake_case is the alternate.
#[derive(Cognomen)]
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
}