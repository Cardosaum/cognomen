use cognomen::{Cognomen, FromLabel, Label};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum ChannelKind {
    Trades,
    L2Book,
    #[cognomen(unknown)]
    Other,
}

fn main() {
    assert_eq!(ChannelKind::from_label("trades").unwrap(), ChannelKind::Trades);
    assert_eq!(ChannelKind::from_label("nope").unwrap(), ChannelKind::Other);
    assert_eq!(ChannelKind::Other.label(), "other");
    assert!(ChannelKind::Other != "nope");
}
