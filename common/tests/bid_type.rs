use common::*;

#[test]
fn bid_type_order() {
    let bid_type1 = BidType::NoTrump;
    let bid_type2 = BidType::Trump(Suit::Spades);
    let bid_type3 = BidType::Trump(Suit::Hearts);

    assert!(bid_type1 > bid_type2);
    assert!(bid_type2 < bid_type1);
    assert!(bid_type3 < bid_type2);
    assert!(bid_type2 > bid_type3);
}