use common::*;
use core::cmp::Ordering;

#[test]
fn card_partial_ord() {
    let card1 = Card::new(Rank::Two, Suit::Spades);
    let card2 = Card::new(Rank::Three, Suit::Spades);
    let card3 = Card::new(Rank::Four, Suit::Hearts);

    assert_eq!(card1 < card2, true);
    assert_eq!(card2 < card1, false);
    assert_eq!(card1 > card2, false);
    assert_eq!(card2 > card1, true);

    assert_eq!(card1 < card3, false);
    assert_eq!(card1 > card3, false);
    assert_eq!(card3 < card1, false);
    assert_eq!(card3 > card1, false);
}

#[test]
fn card_compare_with_trump() {
    let card1 = Card::new(Rank::Two, Suit::Spades);
    let card2 = Card::new(Rank::Three, Suit::Spades);
    let card3 = Card::new(Rank::Four, Suit::Hearts);

    // 2S < 3S, 3S > 2S - No matter the Trump
    assert_eq!(card1.compare_with_trump(&card2, &Bid::new(3, BidType::Trump(Suit::Hearts)).expect("Creating bid: 3 Hearts")), Some(Ordering::Less));
    assert_eq!(card2.compare_with_trump(&card1, &Bid::new(3, BidType::Trump(Suit::Hearts)).expect("Creating bid: 3 Hearts")), Some(Ordering::Greater));
    assert_eq!(card1.compare_with_trump(&card2, &Bid::new(3, BidType::Trump(Suit::Spades)).expect("Creating bid: 3 Spades")), Some(Ordering::Less));
    assert_eq!(card2.compare_with_trump(&card1, &Bid::new(3, BidType::Trump(Suit::Spades)).expect("Creating bid: 3 Spades")), Some(Ordering::Greater));
    assert_eq!(card1.compare_with_trump(&card2, &Bid::new(3, BidType::NoTrump).expect("Creating bid: 3 No Trump")), Some(Ordering::Less));
    assert_eq!(card2.compare_with_trump(&card1, &Bid::new(3, BidType::NoTrump).expect("Creating bid: 3 No Trump")), Some(Ordering::Greater));

    // No Trump - incomparable
    // 2S > 4H, 4H < 2S - Trump Spades
    // 2S < 4H, 4H > 2S - Trump Hearts
    assert_eq!(card1.compare_with_trump(&card3, &Bid::new(3, BidType::NoTrump).expect("Creating bid: 3 No Trump")), None);
    assert_eq!(card3.compare_with_trump(&card1, &Bid::new(3, BidType::NoTrump).expect("Creating bid: 3 No Trump")), None);
    assert_eq!(card1.compare_with_trump(&card3, &Bid::new(3, BidType::Trump(Suit::Spades)).expect("Creating bid: 3 Spades")), Some(Ordering::Greater));
    assert_eq!(card3.compare_with_trump(&card1, &Bid::new(3, BidType::Trump(Suit::Spades)).expect("Creating bid: 3 Spades")), Some(Ordering::Less));
    assert_eq!(card1.compare_with_trump(&card3, &Bid::new(3, BidType::Trump(Suit::Hearts)).expect("Creating bid: 3 Hearts")), Some(Ordering::Less));
    assert_eq!(card3.compare_with_trump(&card1, &Bid::new(3, BidType::Trump(Suit::Hearts)).expect("Creating bid: 3 Hearts")), Some(Ordering::Greater));
}