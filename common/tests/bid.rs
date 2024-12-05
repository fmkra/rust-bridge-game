use common::*;

#[test]
fn bid_new() {
    let bid1 = Bid::new(7, BidType::Trump(Suit::Spades));
    let bid2 = Bid::new(3, BidType::NoTrump);
    let bid3_op = Bid::new(15, BidType::NoTrump);

    let Some(play1) = bid1 else {
        panic!("Bid1 is not Some()");
    };

    assert!(matches!(play1, Bid::Play(7, BidType::Trump(Suit::Spades))));

    let Some(play2) = bid2 else {
        panic!("Bid2 is not Some()");
    };

    assert!(matches!(play2, Bid::Play(3, BidType::NoTrump)));

    assert_eq!(bid3_op, None);
}

#[test]
fn bid_order() {
    let bid1 = Bid::new(2, BidType::Trump(Suit::Spades)).expect("Create Bid: 2 Spades");
    let bid2 = Bid::new(2, BidType::NoTrump).expect("Create Bid: 2 No Trump");
    let bid3 = Bid::new(3, BidType::Trump(Suit::Clubs)).expect("Create Bid: 3 of Clubs");

    assert!(bid1 < bid2);
    assert!(bid2 > bid1);
    assert!(bid2 < bid3);
    assert!(bid3 > bid2);
}