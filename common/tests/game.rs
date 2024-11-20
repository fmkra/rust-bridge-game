use common::*;

#[test]
fn game_trick_max() {
    let mut game = Game::new(
        Bid::new(
            4, 
            BidType::Trump(Suit::Spades)
        ).expect("Bid of number 4 and Spades trump is valid")
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
        ).expect("Bid of number 6 and no trump is valid")
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
        ).expect("Bid of number 6 and herat trump is valid")
    );

    assert_eq!(game3.trick_max(), None);
}