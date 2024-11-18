use common::*;

#[test]
fn bid_new() {
    let bid1 = Bid::new(7, BidType::Trump(Suit::Spades)).expect("Bid of number 7 and Spades Trump is valid");
    let bid2 = Bid::new(3, BidType::NoTrump).expect("Bid of number 3 and no Trump is valid");
    let bid3_op = Bid::new(15, BidType::NoTrump);

    assert_eq!(bid1.number, 7);
    assert_eq!(bid1.typ, BidType::Trump(Suit::Spades));    

    assert_eq!(bid2.number, 3);
    assert_eq!(bid2.typ, BidType::NoTrump);

    assert!(bid3_op.is_none());
}

#[test]
fn bid_order() {
    let bid1 = Bid::new(2, BidType::Trump(Suit::Spades)).expect("Bid of number 2 and Spades Trump is valid");
    let bid2 = Bid::new(2, BidType::NoTrump).expect("Bid of number 2 and no Trump is valid");
    let bid3 = Bid::new(3, BidType::Trump(Suit::Clubs)).expect("Bid of number 3 and Clubs Trump is valid");

    assert!(bid1 < bid2);
    assert!(bid2 > bid1);
    assert!(bid2 < bid3);
    assert!(bid3 > bid2);
}