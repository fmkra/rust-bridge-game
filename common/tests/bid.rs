use common::*;

#[test]
fn bid_new() {
    let bid1 = Bid::new(7, BidType::Trump(Suit::Spades));
    let bid2 = Bid::new(3, BidType::NoTrump);
    let bid3 = Bid::new(15, BidType::NoTrump);

    assert_eq!(bid1.number, 7);
    assert_eq!(bid1.typ, BidType::Trump(Suit::Spades));    

    assert_eq!(bid2.number, 3);
    assert_eq!(bid2.typ, BidType::NoTrump);

    assert_eq!(bid3.number, 0);
    assert_eq!(bid3.typ, BidType::Pass);
}

#[test]
fn bid_order() {
    let bid1 = Bid::new(2, BidType::Trump(Suit::Spades));
    let bid2 = Bid::new(2, BidType::NoTrump);
    let bid3 = Bid::new(3, BidType::Trump(Suit::Clubs));

    assert!(bid1 < bid2);
    assert!(bid2 > bid1);
    assert!(bid2 < bid3);
    assert!(bid3 > bid2);
}