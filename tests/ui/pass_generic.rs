use cognomen::Cognomen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Cognomen)]
#[cognomen(snake_case)]
enum Flag<const N: usize> {
    LeftHand,
    RightHand,
}

fn main() {
    assert_eq!(Flag::<1>::LeftHand.as_str(), "left_hand");
    assert_eq!(
        Flag::<7>::from_label("right_hand").unwrap(),
        Flag::<7>::RightHand
    );
    assert_eq!("left_hand".parse::<Flag<2>>().unwrap(), Flag::<2>::LeftHand);
}
