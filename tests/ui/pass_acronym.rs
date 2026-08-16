use cognomen::{Cognomen, FromLabel, Label};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Wire {
    HTTPResponse,
    Utf8,
    IPv4,
}

fn main() {
    assert_eq!(Wire::HTTPResponse.as_str(), "http_response");
    assert_eq!(Wire::Utf8.as_str(), "utf8");
    assert_eq!(Wire::IPv4.as_str(), "i_pv4");
    assert_eq!(Wire::from_label("http_response").unwrap(), Wire::HTTPResponse);
}
