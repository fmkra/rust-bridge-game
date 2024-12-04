use common::*;

#[test]
fn game_trick_max() {
    let mut game = Game::new(
        Bid::new(
            4, 
            BidType::Trump(Suit::Spades)
        ).expect("Failed to create the bid")
    );

    game.add_card(Card::new(Rank::Three, Suit::Spades));
    game.add_card(Card::new(Rank::Five, Suit::Diamonds));
    game.add_card(Card::new(Rank::King, Suit::Clubs));
    game.add_card(Card::new(Rank::Queen, Suit::Spades));

    assert_eq!(game.trick_max(), Some(&Card::new(Rank::Queen, Suit::Spades)));

    // -------------------------------------

    let mut game2 = Game::new(
        Bid::new(
            6,
            BidType::NoTrump
        ).expect("Failed to create the bid")
    );

    game2.add_card(Card::new(Rank::Two, Suit::Diamonds));
    game2.add_card(Card::new(Rank::Five, Suit::Hearts));
    game2.add_card(Card::new(Rank::Queen, Suit::Spades));
    game2.add_card(Card::new(Rank::Seven, Suit::Clubs));

    assert_eq!(game2.trick_max(), Some(&Card::new(Rank::Two, Suit::Diamonds)));

    let game3 = Game::new(
        Bid::new(
            3,
            BidType::Trump(Suit::Hearts)
        ).expect("Failed to create the bid")
    );

    assert_eq!(game3.trick_max(), None);
}

#[test]
fn game_deal_cards() {
    let game = Game::new(
        Bid::new(
            4, 
            BidType::Trump(Suit::Spades)
        ).expect("Failed to create the bid")
    );

    let hands = game.deal_cards();
    let mut cards : Vec<_> = hands.into_iter().flatten().collect();
    cards.sort_by(|a, b| {
        if a.suit == b.suit {
            return a.rank.cmp(&b.rank);
        }

        return a.suit.cmp(&b.suit)
    });

    let mut cards_iter = cards.iter();
    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades]{
        for rank in 2..15 {
            let card = Card::new(Rank::from_u8(rank).unwrap(), suit);
            assert_eq!(card, *cards_iter.next().unwrap());
        }
    }
}