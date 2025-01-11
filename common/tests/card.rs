use common::*;
use core::cmp::Ordering;

#[test]
fn suit_is_major() {
    let suit1 = Suit::Clubs;
    let suit2 = Suit::Diamonds;
    let suit3 = Suit::Hearts;
    let suit4 = Suit::Spades;

    assert!(!suit1.is_major());
    assert!(!suit2.is_major());
    assert!(suit3.is_major());
    assert!(suit4.is_major());
}

#[test]
fn card_partial_ord() {
    let card1 = Card::new(Rank::Two, Suit::Spades);
    let card2 = Card::new(Rank::Three, Suit::Spades);
    let card3 = Card::new(Rank::Four, Suit::Hearts);

    assert!(card1 < card2);
    assert!(card2 >= card1);
    assert!(card1 <= card2);
    assert!(card2 > card1);

    assert!(card1 >= card3);
    assert!(card1 <= card3);
    assert!(card3 >= card1);
    assert!(card3 <= card1);
}

#[test]
fn card_compare_with_trump() {
    let card1 = Card::new(Rank::Two, Suit::Spades);
    let card2 = Card::new(Rank::Three, Suit::Spades);
    let card3 = Card::new(Rank::Four, Suit::Hearts);

    // 2S < 3S, 3S > 2S - No matter the Trump
    assert_eq!(
        card1.compare_with_trump(&card2, &BidType::Trump(Suit::Hearts)),
        Some(Ordering::Less)
    );
    assert_eq!(
        card2.compare_with_trump(&card1, &BidType::Trump(Suit::Hearts)),
        Some(Ordering::Greater)
    );
    assert_eq!(
        card1.compare_with_trump(&card2, &BidType::Trump(Suit::Spades)),
        Some(Ordering::Less)
    );
    assert_eq!(
        card2.compare_with_trump(&card1, &BidType::Trump(Suit::Spades)),
        Some(Ordering::Greater)
    );
    assert_eq!(
        card1.compare_with_trump(&card2, &BidType::NoTrump),
        Some(Ordering::Less)
    );
    assert_eq!(
        card2.compare_with_trump(&card1, &BidType::NoTrump),
        Some(Ordering::Greater)
    );

    // No Trump - incomparable
    // 2S > 4H, 4H < 2S - Trump Spades
    // 2S < 4H, 4H > 2S - Trump Hearts
    assert_eq!(card1.compare_with_trump(&card3, &BidType::NoTrump), None);
    assert_eq!(card3.compare_with_trump(&card1, &BidType::NoTrump), None);
    assert_eq!(
        card1.compare_with_trump(&card3, &BidType::Trump(Suit::Spades)),
        Some(Ordering::Greater)
    );
    assert_eq!(
        card3.compare_with_trump(&card1, &BidType::Trump(Suit::Spades)),
        Some(Ordering::Less)
    );
    assert_eq!(
        card1.compare_with_trump(&card3, &BidType::Trump(Suit::Hearts)),
        Some(Ordering::Less)
    );
    assert_eq!(
        card3.compare_with_trump(&card1, &BidType::Trump(Suit::Hearts)),
        Some(Ordering::Greater)
    );
}
