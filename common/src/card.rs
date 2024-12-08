use crate::bid::BidType;
use std::cmp::Ordering;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    pub fn compare_with_trump(&self, other: &Card, bid_type: &BidType) -> Option<Ordering> {
        match bid_type {
            BidType::NoTrump => self.partial_cmp(&other),
            BidType::Trump(trump_suit) => {
                if self.suit == *trump_suit && other.suit != *trump_suit {
                    Some(Ordering::Greater)
                } else if self.suit != *trump_suit && other.suit == *trump_suit {
                    Some(Ordering::Less)
                } else {
                    self.partial_cmp(&other)
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
