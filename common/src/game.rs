use crate::bid::Bid;
use crate::card::{Card, Rank, Suit};
use crate::player::Player;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum GameState {
    WaitingForPlayers,
    Auction,
    Tricking,
    Finished,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum BidError {
    GameStateMismatch,
    PlayerOutOfTurn,
    WrongBid,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum BidStatus {
    Auction,
    Tricking,
    Finished,
    Error(BidError),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TrickError {
    GameStateMismatch,
    PlayerOutOfTurn,
    CardNotFound,
    WrongCardSuit,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TrickState {
    pub game_state: GameState,
    pub cards: Vec<Card>,
    pub taker: Player,
}

impl TrickState {
    pub fn new(game_state: GameState, cards: Vec<Card>, taker: Player) -> TrickState {
        TrickState {
            game_state,
            cards,
            taker,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TrickStatus {
    TrickInProgress,
    TrickFinished(TrickState),
    Error(TrickError),
}

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug)]
pub struct GameResult {
    pub bidded: Bid,
    pub won_tricks: usize,
    pub contract_succeeded: bool,
}

#[derive(Debug)]
pub struct Game {
    pub state: GameState,
    pub max_bid: Bid,
    pub max_bidder: Player,
    pub current_player: Player,
    pub player_cards: [Vec<Card>; 4],
    pub collected_cards: [Vec<Card>; 4],
    pub trick_no: u8,
    pub current_trick: Vec<Card>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            state: GameState::WaitingForPlayers,
            current_player: Player::North,
            max_bid: Bid::Pass,
            max_bidder: Player::North,
            player_cards: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            collected_cards: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            trick_no: 0,
            current_trick: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        let mut deck: Vec<Card> = (2..=14)
            .filter_map(Rank::from_u8)
            .flat_map(|rank| {
                [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades]
                    .iter()
                    .map(move |&suit| Card::new(rank, suit))
            })
            .collect();

        let mut rng = rand::thread_rng();
        deck.shuffle(&mut rng);

        self.player_cards = deck
            .chunks(13)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(); // There's always a way to split 52 cards into 4*13

        self.state = GameState::Auction;
    }

    pub fn place_bid(&mut self, player: &Player, bid: Bid) -> BidStatus {
        if self.state != GameState::Auction {
            return BidStatus::Error(BidError::GameStateMismatch);
        }
        if self.current_player != *player {
            return BidStatus::Error(BidError::PlayerOutOfTurn);
        }
        match bid {
            Bid::Pass => {
                self.current_player = self.current_player.next();
                if self.current_player == self.max_bidder {
                    match self.max_bid {
                        Bid::Pass => {
                            self.state = GameState::Finished;
                            return BidStatus::Finished;
                        }
                        _ => {
                            self.state = GameState::Tricking;
                            // Calling next again, as the first player is next of max_bidder
                            self.current_player = self.current_player.next();
                            return BidStatus::Tricking;
                        }
                    }
                }
                return BidStatus::Auction;
            }
            Bid::Play(_, _) => {
                if bid > self.max_bid {
                    self.max_bid = bid;
                    self.max_bidder = *player;
                    self.current_player = self.current_player.next();
                    BidStatus::Auction
                } else {
                    BidStatus::Error(BidError::WrongBid)
                }
            }
        }
    }

    pub fn trick(&mut self, player: &Player, card: &Card) -> TrickStatus {
        if self.state != GameState::Tricking {
            return TrickStatus::Error(TrickError::GameStateMismatch);
        }
        if self.current_player != *player {
            return TrickStatus::Error(TrickError::PlayerOutOfTurn);
        }
        if !self.has_card(player, card) {
            return TrickStatus::Error(TrickError::CardNotFound);
        }
        let player_usize = player.to_usize();

        // Player played the wrong suit, while right suit cards in hand
        if !self.current_trick.is_empty()
            && card.suit != self.current_trick[0].suit
            && self.find_suit(player, &self.current_trick[0].suit)
        {
            return TrickStatus::Error(TrickError::WrongCardSuit);
        }

        // Either the trick is empty, the suit is right,
        // or the player has no more cards of this suit
        self.current_trick.push(card.clone());
        self.player_cards[player_usize].retain(|&c| c != *card);
        self.current_player = self.current_player.next();

        if self.current_trick.len() == 4 {
            // This sets the current player as the winner of the trick
            self.set_winner();
            let winner_usize = self.current_player.to_usize();
            let full_trick = self.current_trick.clone();
            self.collected_cards[winner_usize].append(&mut self.current_trick);
            self.trick_no += 1;

            if self.trick_no == 13 {
                return TrickStatus::TrickFinished(TrickState::new(
                    GameState::Finished,
                    full_trick,
                    self.current_player.clone(),
                ));
            }

            return TrickStatus::TrickFinished(TrickState::new(
                GameState::Tricking,
                full_trick,
                self.current_player.clone(),
            ));
        }

        TrickStatus::TrickInProgress
    }

    pub fn evaluate(&self) -> Option<GameResult> {
        if self.state != GameState::Finished {
            return None;
        }
        let bidded = self.max_bid.clone();
        match bidded {
            Bid::Pass => None,
            Bid::Play(val, _) => {
                let max_bidder = self.max_bidder.to_usize();
                let dummy = self.max_bidder.next().next().to_usize();
                let won_tricks: usize = self.collected_cards[max_bidder].len() / 4
                    + self.collected_cards[dummy].len() / 4;
                let contract_succeeded: bool = (won_tricks - 6) >= val.into();
                Some(GameResult {
                    bidded,
                    won_tricks,
                    contract_succeeded,
                })
            }
        }
    }

    pub fn get_dummy(&self) -> Option<&Vec<Card>> {
        if self.state != GameState::Tricking {
            return None;
        }
        if self.trick_no == 0 && self.current_trick.len() == 0 {
            // No cards were given, the dummy doesn't reveal its' cards yet.
            return None;
        }
        let dummy_usize = self.max_bidder.next().next().to_usize();
        return Some(&self.player_cards[dummy_usize]);
    }

    pub fn get_cards(&self, player: &Player) -> &Vec<Card> {
        let player_usize = player.to_usize();
        return &self.player_cards[player_usize];
    }

    pub fn has_card(&self, player: &Player, card: &Card) -> bool {
        let player_usize = player.to_usize();
        self.player_cards[player_usize]
            .iter()
            .find(|c| *c == card)
            .is_some()
    }

    pub fn find_suit(&self, player: &Player, suit: &Suit) -> bool {
        let player_usize = player.to_usize();
        self.player_cards[player_usize]
            .iter()
            .find(|c| c.suit == *suit)
            .is_some()
    }

    pub fn set_winner(&mut self) {
        let max_card = self.trick_max();
        // Unwrap is valid, as max_card exists in self.current_trick
        let id = self
            .current_trick
            .iter()
            .position(|c| c == max_card)
            .unwrap();
        self.current_player = self.current_player.skip(id);
    }

    // This function is to be called only after the Auction phase of the game
    // and when the trick is not empty. Otherwise, the unwrap() will cause a panic!
    pub fn trick_max(&self) -> &Card {
        // As the game is in Tricking state, try_into() will always return Some(BidType)
        let bid_type = self.max_bid.try_into().unwrap();
        self.current_trick
            .iter()
            .max_by(|&cur, &card| {
                cur.compare_with_trump(card, &bid_type)
                    .unwrap_or(std::cmp::Ordering::Greater)
            })
            .unwrap()
    }
}
