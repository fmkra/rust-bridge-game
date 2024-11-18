use common::*;

#[test]
fn rank_from() {
    let rank1 = Rank::from_u8(7);
    let rank2 = Rank::from_u8(14);
    let rank3 = Rank::from_u8(69);

    assert_eq!(rank1, Some(Rank::Seven));
    assert_eq!(rank2, Some(Rank::Ace));
    assert!(rank3.is_none());
}

#[test]
fn rank_to() {
    let rank1 = Rank::Ace;
    let rank2 = Rank::Seven;

    assert_eq!(rank1.to_u8(), 14);
    assert_eq!(rank2.to_u8(), 7);   
}