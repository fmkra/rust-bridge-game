use common::*;

#[test]
fn game_trick_max() {
    let mut game1 = Game::new();
    game1.max_bid = Bid::new(2, BidType::Trump(Suit::Spades)).expect("Create bid: 2 Spades");

    game1.add_card(Card::new(Rank::Three, Suit::Spades));
    game1.add_card(Card::new(Rank::Five, Suit::Diamonds));
    game1.add_card(Card::new(Rank::King, Suit::Clubs));
    game1.add_card(Card::new(Rank::Queen, Suit::Spades));

    assert_eq!(game1.trick_max(), Some(&Card::new(Rank::Queen, Suit::Spades)));

    // -------------------------------------

    let mut game2 = Game::new();
    game2.max_bid = Bid::new(6, BidType::NoTrump).expect("Create bid: 6 No Trump");

    game2.add_card(Card::new(Rank::Two, Suit::Diamonds));
    game2.add_card(Card::new(Rank::Five, Suit::Hearts));
    game2.add_card(Card::new(Rank::Queen, Suit::Spades));
    game2.add_card(Card::new(Rank::Seven, Suit::Clubs));

    assert_eq!(game2.trick_max(), Some(&Card::new(Rank::Two, Suit::Diamonds)));

    // ------------------------------------- 
 
    let mut game3 = Game::new();
    game3.max_bid = Bid::new(2, BidType::NoTrump).expect("Create bid: 2 No Trump");

    assert_eq!(game3.trick_max(), None);
}

#[test]
fn game_deal_cards() {
    let game = Game::new();

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