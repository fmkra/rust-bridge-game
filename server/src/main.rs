use common::{Bid, BidType, Card, Rank, Suit};

fn main() {
    let card1 = Card::new(Rank::from_u8(7).unwrap(), Suit::Spades);
    let card2 = Card::new(Rank::from_u8(12).unwrap(), Suit::Clubs);
    let card3 = Card::new(Rank::Queen, Suit::Clubs);
    let bid_invalid = Bid::new(8, BidType::Trump(Suit::Spades));
    let bid_valid = Bid::new(3, BidType::Trump(Suit::Spades));

    assert!(card2 == card3);
    assert!(bid_invalid.is_none());
    assert!(bid_valid.is_some());
    assert_eq!(card1.compare_with_trump(&card2, BidType::Trump(Suit::Spades)), Some(std::cmp::Ordering::Greater));
}
