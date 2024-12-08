use crate::bid::Bid;
use crate::card::{Card, Rank, Suit};
use crate::player::Player;
use rand::prelude::SliceRandom;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum GameState {
    WaitingForPlayers,
    Auction,
    Tricking,
    Finished,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum GameError {
    GameStateMismatch,
    PlayerOutOfTurn,
    WrongBid,
    WrongTrick,
    CardNotFound,
    WrongCardSuit,
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

    pub fn place_bid(&mut self, player: &Player, bid: Bid) -> Result<GameState, GameError> {
        if self.state != GameState::Auction {
            return Err(GameError::GameStateMismatch);
        }
        if self.current_player != *player {
            return Err(GameError::PlayerOutOfTurn);
        }
        match bid {
            Bid::Pass => {
                self.current_player = self.current_player.next();
                if self.current_player == self.max_bidder {
                    match self.max_bid {
                        Bid::Pass => {
                            self.state = GameState::Finished;
                        }
                        _ => {
                            self.state = GameState::Tricking;
                        }
                    }
                }
                Ok(self.state)
            }
            Bid::Play(_, _) => {
                if bid > self.max_bid {
                    self.max_bid = bid;
                    self.max_bidder = *player;
                    self.current_player = self.current_player.next();
                    Ok(self.state)
                } else {
                    Err(GameError::WrongBid)
                }
            }
        }
    }

    pub fn trick(&mut self, player: &Player, card: &Card) -> Result<GameState, GameError> {
        if self.state != GameState::Tricking {
            return Err(GameError::GameStateMismatch);
        }
        if self.current_player != *player {
            return Err(GameError::PlayerOutOfTurn);
        }
        if !self.has_card(player, card) {
            return Err(GameError::CardNotFound);
        }
        let player_usize = player.to_usize();

        // Player played the wrong suit, while right suit cards in hand
        if !self.current_trick.is_empty()
            && card.suit != self.current_trick[0].suit
            && self.find_suit(player, &self.current_trick[0].suit)
        {
            return Err(GameError::WrongCardSuit);
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
            self.collected_cards[winner_usize].append(&mut self.current_trick);
            self.trick_no += 1;

            if self.trick_no == 13 {
                return Ok(GameState::Finished);
            }
        }

        Ok(GameState::Tricking)
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
