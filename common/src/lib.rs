use core::cmp::Ordering;
use rand::prelude::SliceRandom;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl Rank {
    pub fn from_u8(value: u8) -> Option<Rank> {
        match value {
            2 => Some(Rank::Two),
            3 => Some(Rank::Three),
            4 => Some(Rank::Four),
            5 => Some(Rank::Five),
            6 => Some(Rank::Six),
            7 => Some(Rank::Seven),
            8 => Some(Rank::Eight),
            9 => Some(Rank::Nine),
            10 => Some(Rank::Ten),
            11 => Some(Rank::Jack),
            12 => Some(Rank::Queen),
            13 => Some(Rank::King),
            14 => Some(Rank::Ace),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BidType {
    Trump(Suit),
    NoTrump,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Bid {
    Pass,
    Play(u8, BidType),
}

impl Bid {
    pub fn new(number: u8, typ: BidType) -> Option<Bid> {
        if number >= 1 && number <= 7 {
            Some(Bid::Play(number, typ))
        } else {
            None
        }
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Bid::Pass, Bid::Pass) => Ordering::Equal,
            (Bid::Pass, Bid::Play(_, _)) => Ordering::Less,
            (Bid::Play(_, _), Bid::Pass) => Ordering::Greater,
            (Bid::Play(self_number, self_type), Bid::Play(other_number, other_type)) => {
                match self_number.cmp(&other_number) {
                    Ordering::Equal => self_type.cmp(&other_type),
                    other => other,
                }
            }
        }
    }
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    pub fn compare_with_trump(&self, other: &Card, bid: &Bid) -> Option<Ordering> {
        match bid {
            Bid::Pass => None,
            Bid::Play(_, bid_type) => match bid_type {
                BidType::NoTrump => self.partial_cmp(&other),
                BidType::Trump(trump_suit) => {
                    if self.suit == *trump_suit && other.suit != *trump_suit {
                        Some(Ordering::Greater)
                    } else if self.suit != *trump_suit && other.suit == *trump_suit {
                        Some(Ordering::Less)
                    } else {
                        self.partial_cmp(&other)
                    }
                }
            },
        }
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.suit != other.suit {
            None
        } else {
            self.rank.partial_cmp(&other.rank)
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum GameState {
    WaitingForPlayers,
    Bidding,
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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Player {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Player {
    pub fn next(&self) -> Player {
        self.skip(1)
    }

    pub fn skip(&self, num_skips: usize) -> Player {
        // Unwrap() is valid, as the number is in [0; 3]
        Player::from_usize((self.to_usize() + num_skips) % 4).unwrap()
    }

    pub fn from_u8(value: u8) -> Option<Player> {
        match value {
            0 => Some(Player::North),
            1 => Some(Player::East),
            2 => Some(Player::South),
            3 => Some(Player::West),
            _ => None,
        }
    }

    pub fn from_usize(value: usize) -> Option<Player> {
        match value.try_into() {
            Ok(value_u8) => Player::from_u8(value_u8),
            Err(_) => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
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
    
    pub fn start(&mut self) -> Result<GameState, GameError>{
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
            .take(4)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(); // There's always a way to split 52 cards into 4*13

        self.state = GameState::Bidding;

        Ok(GameState::Bidding)
    }
    
    pub fn place_bid(&mut self, player: &Player, bid: Bid) -> Result<GameState, GameError> {
        if self.state != GameState::Bidding {
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
        if !self.find_card(player, card) {
            return Err(GameError::CardNotFound);
        }
        let player_usize = player.to_usize();
        
        // Player played the wrong suit, while right suit cards in hand
        if !self.current_trick.is_empty() &&
            card.suit != self.current_trick[0].suit &&
            self.find_suit(player, &self.current_trick[0].suit) {
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

    pub fn add_card(&mut self, c1: Card) {
        self.current_trick.push(c1);
    }

    pub fn find_card(&self, player: &Player, card: &Card) -> bool {
        let player_usize = player.to_usize();
        self.player_cards[player_usize].iter().find(|c| *c == card).is_some()
    }
    
    pub fn find_suit(&self, player: &Player, suit: &Suit) -> bool {
        let player_usize = player.to_usize();
        self.player_cards[player_usize].iter().find(|c| c.suit == *suit).is_some()
    }

    pub fn set_winner(&mut self) {
        let max_card = self.trick_max();
        // Unwrap is valid, as max_card exists in self.current_trick
        let id = self.current_trick.iter().position(|c| c == max_card).unwrap();
        self.current_player = self.current_player.skip(id);
    }

    // This function is to be called only after the bidding phase of the game
    // and when the trick is not empty. Otherwise, the unwrap() will cause a panic!
    pub fn trick_max(&self) -> &Card {
        self.current_trick.iter().max_by(|&cur, &card| {
            cur.compare_with_trump(card, &self.max_bid)
                .unwrap_or(std::cmp::Ordering::Greater)
        }).unwrap()
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
