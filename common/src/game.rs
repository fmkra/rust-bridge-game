use crate::bid::Bid;
use crate::card::{Card, Rank, Suit};
use crate::player::Player;
use crate::BidType;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum GameState {
    WaitingForPlayers,
    Auction,
    Tricking,
    Finished,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum GameValue {
    Regular,
    Doubled,
    Redoubled,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum BidError {
    GameStateMismatch,
    PlayerOutOfTurn,
    WrongBid,
    CantDouble,
    CantRedouble,
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
    pub points: [usize; 4],
    pub cards: Vec<Card>,
    pub taker: Player,
}

impl TrickState {
    pub fn new(game_state: GameState, points: [usize; 4], cards: Vec<Card>, taker: Player) -> TrickState {
        TrickState {
            game_state,
            points,
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
    pub game_value: GameValue,
    pub max_bidder: Player,
    pub first_bidder: Player,
    pub current_player: Player,
    pub player_cards: [Vec<Card>; 4],
    pub collected_cards: [Vec<Card>; 4],
    pub points: [usize; 4],
    pub game_wins: [usize; 4],
    pub vulnerable: [bool; 4],
    pub trick_no: u8,
    pub current_trick: Vec<Card>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            state: GameState::WaitingForPlayers,
            current_player: Player::North,
            max_bid: Bid::Pass,
            game_value: GameValue::Regular,
            max_bidder: Player::North,
            first_bidder: Player::North,
            player_cards: Default::default(),
            collected_cards: Default::default(),
            points: Default::default(),
            game_wins: Default::default(),
            vulnerable: Default::default(), // Default as [false, false, false, false]
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
                BidStatus::Auction
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
            Bid::Double => {
                if self.max_bid == Bid::Pass || !player.is_opponent(self.max_bidder) {
                    return BidStatus::Error(BidError::CantDouble);
                }

                self.max_bidder = self.current_player;
                self.current_player = self.current_player.next();
                self.game_value = GameValue::Doubled;

                BidStatus::Auction
            }
            Bid::Redouble => {
                if self.max_bid == Bid::Pass
                    || !player.is_opponent(self.max_bidder)
                    || self.game_value != GameValue::Doubled
                {
                    return BidStatus::Error(BidError::CantRedouble);
                }

                self.max_bidder = self.current_player;
                self.current_player = self.current_player.next();
                self.game_value = GameValue::Redoubled;

                BidStatus::Auction
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
            && self.has_suit(player, &self.current_trick[0].suit)
        {
            return TrickStatus::Error(TrickError::WrongCardSuit);
        }

        // Either the trick is empty, the suit is right,
        // or the player has no more cards of this suit
        self.current_trick.push(*card);
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
                // This function sets the self.state as Finished if the game is finished
                // e.g. any pair has won 2 deals, then the winner is the one who has more points.
                self.distribute_points();

                let taker = self.current_player;

                if self.state != GameState::Finished {
                    self.trick_no = 0;
                    self.state = GameState::Auction;
                    self.game_value = GameValue::Regular;
                    self.first_bidder = self.first_bidder.next();
                    self.current_player = self.first_bidder;
                }
              
                return TrickStatus::TrickFinished(TrickState::new(
                    self.state,
                    self.points,
                    full_trick,
                    taker,
                ));
            }

            return TrickStatus::TrickFinished(TrickState::new(
                GameState::Tricking,
                self.points,
                full_trick,
                self.current_player,
            ));
        }

        TrickStatus::TrickInProgress
    }

    pub fn distribute_points(&mut self) {
        let bidder_usize = self.max_bidder.to_usize();
        let partner_usize = self.max_bidder.get_partner().to_usize();
    
        let tricks_earned = (self.collected_cards[bidder_usize].len()
            + self.collected_cards[partner_usize].len())
            / 4;
    
        // Destructure self.max_bid, as it's guaranteed to be Bid::Play
        let Bid::Play(tricks_declared_value, bid_type) = self.max_bid else {
            eprintln!("self.max_bid should always be Bid::Play when distributing points");
            return;
        };
        let tricks_declared = tricks_declared_value as usize + 6;
    
        let is_vulnerable = self.vulnerable[bidder_usize];
        let contract_succeeded = tricks_earned >= tricks_declared;

        if contract_succeeded {
            // Calculate contract points
            let base_points = match bid_type {
                BidType::Trump(Suit::Clubs) | BidType::Trump(Suit::Diamonds) => {
                    20 * (tricks_declared - 6)
                }
                BidType::Trump(Suit::Hearts) | BidType::Trump(Suit::Spades) => {
                    30 * (tricks_declared - 6)
                }
                BidType::NoTrump => {
                    40 + 30 * (tricks_declared - 7) // First trick 40, subsequent tricks 30
                }
            };
    
            let doubled_bonus = match self.game_value {
                GameValue::Doubled => 50,
                GameValue::Redoubled => 100,
                _ => 0,
            };
    
            self.points[bidder_usize] += base_points;
            self.points[bidder_usize] += doubled_bonus;
     
            // Slam bonuses
            if tricks_declared == 12 {
                self.points[bidder_usize] += if is_vulnerable { 750 } else { 500 };
            } else if tricks_declared == 13 {
                self.points[bidder_usize] += if is_vulnerable { 1500 } else { 1000 };
            }
    
            // Overtrick points
            let overtricks = tricks_earned - tricks_declared;
            let overtrick_points = match self.game_value {
                GameValue::Doubled => {
                    if is_vulnerable {
                        overtricks * 200
                    } else {
                        overtricks * 100
                    }
                }
                GameValue::Redoubled => {
                    if is_vulnerable {
                        overtricks * 400
                    } else {
                        overtricks * 200
                    }
                }
                _ => overtricks * match bid_type {
                    BidType::Trump(Suit::Clubs) | BidType::Trump(Suit::Diamonds) => 20,
                    BidType::Trump(Suit::Hearts) | BidType::Trump(Suit::Spades) => 30,
                    BidType::NoTrump => 30, // Overtricks in NoTrump are 30 points each
                },
            };
    
            self.points[bidder_usize] += overtrick_points;
            
            // Mark the pair as vulnerable if they won the game
            self.vulnerable[bidder_usize] = true;
            self.vulnerable[partner_usize] = true;
    
            // Check if the pair has won two games and end the match if so
            self.game_wins[bidder_usize] += 1;
            self.game_wins[partner_usize] += 1;
    
            if self.game_wins[bidder_usize] >= 2 || self.game_wins[partner_usize] >= 2 {
                self.state = GameState::Finished;
            }
        } else {
            // Penalty points for undertricks
            let undertricks = tricks_declared - tricks_earned;
            let penalty_points = match self.game_value {
                GameValue::Regular => {
                    if is_vulnerable {
                        // 100 per undertrick
                        undertricks * 100
                    } else {
                        // 50 per undertrick
                        undertricks * 50
                    }
                }
                GameValue::Doubled => {
                    if is_vulnerable {
                        // 200 for the first undertrick, 300 for each subsequent
                        200 + (undertricks - 1) * 300
                    } else {
                        // 100 for the first undertrick, 200 for 2nd and 3rd undertrick, 300 for subsequent
                        100 + 
                        ((undertricks - 1 > 0) as usize * 200) +  
                        ((undertricks - 2 > 0) as usize * 200) +  
                        ((undertricks - 3 > 0) as usize * (undertricks - 3) * 300)
                    }
                }
                GameValue::Redoubled => {
                    if is_vulnerable {
                        // 400 for the first undertrick, 600 for each subsequent
                        400 + (undertricks - 1) * 600
                    } else {
                        // 200 for the first undertrick, 400 for 2nd and 3rd undertrick, 600 for subsequent
                        200 + 
                        ((undertricks - 1 > 0) as usize * 400) +  
                        ((undertricks - 2 > 0) as usize * 400) +  
                        ((undertricks - 3 > 0) as usize * (undertricks - 3) * 600)
                    }
                }
            };
    
            let opponents = [
                self.max_bidder.next().to_usize(),
                self.max_bidder.skip(3).to_usize(),
            ];
    
            for &opponent in &opponents {
                self.points[opponent] += penalty_points;
            }
        }
        // Make sure the partner has the same amounts of points as bidder, as the game is played in pairs.
        self.points[partner_usize] = self.points[bidder_usize];
    }
    

    pub fn evaluate(&self) -> Option<GameResult> {
        if self.state != GameState::Finished {
            return None;
        }
        let bidded = self.max_bid;
        match bidded {
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
            _ => None,
        }
    }

    pub fn get_dummy(&self) -> Option<&Vec<Card>> {
        if self.state != GameState::Tricking {
            return None;
        }
        if self.trick_no == 0 && self.current_trick.is_empty() {
            // No cards were given, the dummy doesn't reveal its' cards yet.
            return None;
        }
        let dummy_usize = self.max_bidder.next().next().to_usize();
        Some(&self.player_cards[dummy_usize])
    }

    pub fn get_cards(&self, player: &Player) -> &Vec<Card> {
        let player_usize = player.to_usize();
        &self.player_cards[player_usize]
    }

    pub fn has_card(&self, player: &Player, card: &Card) -> bool {
        let player_usize = player.to_usize();
        self.player_cards[player_usize].iter().any(|c| c == card)
    }

    pub fn has_suit(&self, player: &Player, suit: &Suit) -> bool {
        let player_usize = player.to_usize();
        self.player_cards[player_usize]
            .iter()
            .any(|c| c.suit == *suit)
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
