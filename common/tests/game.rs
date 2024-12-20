use common::*;

#[test]
fn game_trick_max() {
    let mut game1 = Game::new();
    game1.max_bid = Bid::new(2, BidType::Trump(Suit::Spades)).expect("Create bid: 2 Spades");

    game1
        .current_trick
        .push(Card::new(Rank::Three, Suit::Spades));
    game1
        .current_trick
        .push(Card::new(Rank::Five, Suit::Diamonds));
    game1.current_trick.push(Card::new(Rank::King, Suit::Clubs));
    game1
        .current_trick
        .push(Card::new(Rank::Queen, Suit::Spades));

    assert_eq!(game1.trick_max(), &Card::new(Rank::Queen, Suit::Spades));

    // -------------------------------------

    let mut game2 = Game::new();
    game2.max_bid = Bid::new(6, BidType::NoTrump).expect("Create bid: 6 No Trump");

    game2
        .current_trick
        .push(Card::new(Rank::Two, Suit::Diamonds));
    game2
        .current_trick
        .push(Card::new(Rank::Five, Suit::Hearts));
    game2
        .current_trick
        .push(Card::new(Rank::Queen, Suit::Spades));
    game2
        .current_trick
        .push(Card::new(Rank::Seven, Suit::Clubs));

    assert_eq!(game2.trick_max(), &Card::new(Rank::Two, Suit::Diamonds));
}

#[test]
fn game_start() {
    let mut game = Game::new();
    game.start();
    assert_eq!(game.state, GameState::Auction);

    let hands = game.player_cards;
    let mut cards: Vec<_> = hands.into_iter().flatten().collect();
    cards.sort_by(|a, b| {
        if a.suit == b.suit {
            return a.rank.cmp(&b.rank);
        }

        return a.suit.cmp(&b.suit);
    });

    let mut cards_iter = cards.iter();
    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        for rank in 2..=14 {
            let card = Card::new(Rank::from_u8(rank).unwrap(), suit);
            assert_eq!(card, *cards_iter.next().unwrap());
        }
    }
}

#[test]
fn game_place_bid() {
    let mut game = Game::new();

    assert_eq!(
        BidStatus::Error(BidError::GameStateMismatch),
        game.place_bid(
            &Player::North,
            Bid::new(3, BidType::Trump(Suit::Clubs)).unwrap()
        )
    );

    game.state = GameState::Auction;

    assert_eq!(
        BidStatus::Auction,
        game.place_bid(
            &Player::North,
            Bid::new(3, BidType::Trump(Suit::Clubs)).unwrap()
        )
    );
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(
            &Player::East,
            Bid::new(3, BidType::Trump(Suit::Diamonds)).unwrap()
        )
    );
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(
            &Player::South,
            Bid::new(3, BidType::Trump(Suit::Hearts)).unwrap()
        )
    );
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(
            &Player::West,
            Bid::new(3, BidType::Trump(Suit::Spades)).unwrap()
        )
    );
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(&Player::North, Bid::new(3, BidType::NoTrump).unwrap())
    );

    // Player Auction out of his turn.
    assert_eq!(
        BidStatus::Error(BidError::PlayerOutOfTurn),
        game.place_bid(&Player::North, Bid::Pass)
    );

    // Player placing a wrong bid - lower than the current max bid.
    assert_eq!(
        BidStatus::Error(BidError::WrongBid),
        game.place_bid(
            &Player::East,
            Bid::new(2, BidType::Trump(Suit::Spades)).unwrap()
        )
    );

    assert_eq!(BidStatus::Auction, game.place_bid(&Player::East, Bid::Pass));
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(&Player::South, Bid::Pass)
    );
    assert_eq!(
        BidStatus::Tricking,
        game.place_bid(&Player::West, Bid::Pass)
    );

    assert_eq!(Player::East, game.current_player);
}

#[test]
fn game_place_trick() {
    let mut game = Game::new();
    game.max_bid = Bid::new(3, BidType::NoTrump).unwrap();
    game.state = GameState::Tricking;

    let mut cards_north: Vec<Card> = Vec::new();
    let mut cards_east: Vec<Card> = Vec::new();
    let mut cards_south: Vec<Card> = Vec::new();
    let mut cards_west: Vec<Card> = Vec::new();

    // Each of the players has all 2 - 10 cards from 1 suit.
    for rank_u8 in 2..=10 {
        cards_north.push(Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Clubs));
        cards_east.push(Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Diamonds));
        cards_south.push(Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Hearts));
        cards_west.push(Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Spades));
    }

    // North has all the Jacks
    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        cards_north.push(Card::new(Rank::Jack, suit));
    }

    // East has all the Queens
    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        cards_east.push(Card::new(Rank::Queen, suit));
    }

    // South has all the Kings
    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        cards_south.push(Card::new(Rank::King, suit));
    }

    // West has all the Aces
    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        cards_west.push(Card::new(Rank::Ace, suit));
    }

    game.player_cards = [cards_north, cards_east, cards_south, cards_west];

    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::North, &Card::new(Rank::Jack, Suit::Spades))
    );
    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::East, &Card::new(Rank::Queen, Suit::Spades))
    );
    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::South, &Card::new(Rank::King, Suit::Spades))
    );

    let mut trick_state = TrickState::new(
        GameState::Tricking,
        vec![
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Queen, Suit::Spades),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Ace, Suit::Spades),
        ],
        Player::West,
    );

    assert_eq!(
        TrickStatus::TrickFinished(trick_state),
        game.trick(&Player::West, &Card::new(Rank::Ace, Suit::Spades))
    );

    // Tricks are indexed from 0
    assert_eq!(game.trick_no, 1);
    assert!(game.current_trick.is_empty());
    assert_eq!(
        game.collected_cards[Player::to_usize(&Player::West)],
        vec![
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Queen, Suit::Spades),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Ace, Suit::Spades),
        ]
    );

    assert_eq!(
        TrickStatus::Error(TrickError::PlayerOutOfTurn),
        game.trick(&Player::North, &Card::new(Rank::Jack, Suit::Spades))
    );
    assert_eq!(
        TrickStatus::Error(TrickError::CardNotFound),
        game.trick(&Player::West, &Card::new(Rank::Two, Suit::Clubs))
    );

    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::West, &Card::new(Rank::Two, Suit::Spades))
    );
    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::North, &Card::new(Rank::Two, Suit::Clubs))
    );
    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::East, &Card::new(Rank::Two, Suit::Diamonds))
    );

    trick_state = TrickState::new(
        GameState::Tricking,
        vec![
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Two, Suit::Hearts),
        ],
        Player::West,
    );

    assert_eq!(
        TrickStatus::TrickFinished(trick_state),
        game.trick(&Player::South, &Card::new(Rank::Two, Suit::Hearts))
    );

    assert_eq!(game.trick_no, 2);
    assert!(game.current_trick.is_empty());
    assert_eq!(
        game.collected_cards[Player::to_usize(&Player::West)],
        vec![
            Card::new(Rank::Jack, Suit::Spades),
            Card::new(Rank::Queen, Suit::Spades),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Two, Suit::Hearts),
        ]
    );
}

#[test]
fn game_full_game() {
    let mut game = Game::new();
    let mut cards_north: Vec<Card> = Vec::new();
    let mut cards_east: Vec<Card> = Vec::new();
    let mut cards_south: Vec<Card> = Vec::new();
    let mut cards_west: Vec<Card> = Vec::new();

    // Setup mock just as in game.start() as it's random
    // Simple case - all clubs for north etc.
    for rank_u8 in 2..15 {
        cards_north.push(Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Clubs));
        cards_east.push(Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Diamonds));
        cards_south.push(Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Hearts));
        cards_west.push(Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Spades));
    }

    game.player_cards = [cards_north, cards_east, cards_south, cards_west];
    game.state = GameState::Auction;

    // Auction
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(
            &Player::North,
            Bid::new(2, BidType::Trump(Suit::Clubs)).unwrap()
        )
    );
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(
            &Player::East,
            Bid::new(2, BidType::Trump(Suit::Diamonds)).unwrap()
        )
    );
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(
            &Player::South,
            Bid::new(2, BidType::Trump(Suit::Hearts)).unwrap()
        )
    );
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(
            &Player::West,
            Bid::new(3, BidType::Trump(Suit::Clubs)).unwrap()
        )
    );
    assert_eq!(
        BidStatus::Auction,
        game.place_bid(&Player::North, Bid::Pass)
    );
    assert_eq!(BidStatus::Auction, game.place_bid(&Player::East, Bid::Pass));
    assert_eq!(
        BidStatus::Tricking,
        game.place_bid(&Player::South, Bid::Pass)
    );

    // Tricking
    for rank_u8 in 2..14 {
        assert_eq!(
            TrickStatus::TrickInProgress,
            game.trick(
                &Player::North,
                &Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Clubs)
            )
        );
        assert_eq!(
            TrickStatus::TrickInProgress,
            game.trick(
                &Player::East,
                &Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Diamonds)
            )
        );
        assert_eq!(
            TrickStatus::TrickInProgress,
            game.trick(
                &Player::South,
                &Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Hearts)
            )
        );

        let trick_state = TrickState::new(
            GameState::Tricking,
            vec![
                Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Clubs),
                Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Diamonds),
                Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Hearts),
                Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Spades),
            ],
            Player::North,
        );

        assert_eq!(
            TrickStatus::TrickFinished(trick_state),
            game.trick(
                &Player::West,
                &Card::new(Rank::from_u8(rank_u8).unwrap(), Suit::Spades)
            )
        );
    }

    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::North, &Card::new(Rank::Ace, Suit::Clubs))
    );
    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::East, &Card::new(Rank::Ace, Suit::Diamonds))
    );
    assert_eq!(
        TrickStatus::TrickInProgress,
        game.trick(&Player::South, &Card::new(Rank::Ace, Suit::Hearts))
    );

    let trick_state = TrickState::new(
        GameState::Finished,
        vec![
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::Ace, Suit::Diamonds),
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::Ace, Suit::Spades),
        ],
        Player::North,
    );

    assert_eq!(
        TrickStatus::TrickFinished(trick_state),
        game.trick(&Player::West, &Card::new(Rank::Ace, Suit::Spades))
    );

    assert_eq!(
        [
            Vec::<Card>::new(),
            Vec::<Card>::new(),
            Vec::<Card>::new(),
            Vec::<Card>::new()
        ],
        game.player_cards
    );
    assert_eq!(13, game.trick_no);
}

#[test]
fn game_evaluate() {
    let game = Game::new();
    assert_eq!(None, game.evaluate());

    let mut game2 = Game::new();
    game2.max_bid = Bid::new(2, BidType::Trump(Suit::Spades)).expect("Create bid: 2 Spades");
    game2.max_bidder = Player::North;
    game2.state = GameState::Finished;

    // Arbitrary game state
    // North took 6 tricks
    // South took 2 tricks
    // East took 5 tricks

    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        for rank in 2..=7 {
            game2.collected_cards[0].push(Card::new(Rank::from_u8(rank).unwrap(), suit));
        }

        for rank in 8..=12 {
            game2.collected_cards[1].push(Card::new(Rank::from_u8(rank).unwrap(), suit));
        }

        for rank in 13..=14 {
            game2.collected_cards[2].push(Card::new(Rank::from_u8(rank).unwrap(), suit));
        }
    }

    let res = GameResult {
        bidded: game2.max_bid.clone(),
        won_tricks: 8,
        contract_succeeded: true,
    };

    assert_eq!(Some(res), game2.evaluate());
}

#[test]
fn game_get_dummy() {
    let game = Game::new();
    assert_eq!(None, game.get_dummy());

    let mut game2 = Game::new();
    game2.state = GameState::Tricking;
    assert_eq!(None, game2.get_dummy());

    let cards = vec![
        Card::new(Rank::Queen, Suit::Spades),
        Card::new(Rank::King, Suit::Spades),
        Card::new(Rank::Ace, Suit::Clubs),
        Card::new(Rank::Jack, Suit::Spades),
        Card::new(Rank::Two, Suit::Spades),
    ];

    game2.max_bidder = Player::North;
    // Set south player cards
    game2.player_cards[2] = cards.clone();
    game2.trick_no = 0;
    game2.current_trick.push(Card::new(Rank::Ace, Suit::Spades));
    assert_eq!(Some(&cards), game2.get_dummy());
}
