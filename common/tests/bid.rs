use common::*;

#[test]
fn bid_new() {
    let bid1_op = Bid::new(7, BidType::Trump(Suit::Spades));
    let bid2_op = Bid::new(3, BidType::NoTrump);
    let bid3_op = Bid::new(15, BidType::NoTrump);

    assert!(bid1_op.is_some());

    let bid1 = bid1_op.unwrap();
    assert_eq!(bid1.number, 7);
    assert_eq!(bid1.typ, BidType::Trump(Suit::Spades));    

    assert!(bid2_op.is_some());

    let bid2 = bid2_op.unwrap();
    assert_eq!(bid2.number, 3);
    assert_eq!(bid2.typ, BidType::NoTrump);

    assert!(bid3_op.is_none());
}

#[test]
fn bid_order() {
    let bid1_op = Bid::new(2, BidType::Trump(Suit::Spades));
    let bid2_op = Bid::new(2, BidType::NoTrump);
    let bid3_op = Bid::new(3, BidType::Trump(Suit::Clubs));

    assert!(bid1_op.is_some());
    assert!(bid2_op.is_some());
    assert!(bid3_op.is_some());

    let bid1 = bid1_op.unwrap();
    let bid2 = bid2_op.unwrap();
    let bid3 = bid3_op.unwrap();

    assert!(bid1 < bid2);
    assert!(bid2 > bid1);
    assert!(bid2 < bid3);
    assert!(bid3 > bid2);
}