use core::cmp::Ordering;

#[repr(u8)]
#[derive(PartialEq, PartialOrd)]
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

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(PartialEq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(PartialEq)]
pub enum BidType {
    NoTrump,
    Trump(Suit),
}

pub struct Bid {
    number: u8,
    typ: BidType,
}

impl Bid {
    pub fn new(number: u8, typ: BidType) -> Option<Bid> {
        if number >= 1 && number <= 7 {
            Some(Bid { number, typ })
        } else {
            None
        }
    }
}

#[derive(PartialEq)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
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

impl Card {
    pub fn compare_with_trump(
        &self,
        other: &Card,
        bid_type: BidType
    ) -> Option<std::cmp::Ordering> {
        match bid_type {
            BidType::NoTrump => self.rank.partial_cmp(&other.rank),
            BidType::Trump(trump_suit) => {
                if self.suit == trump_suit && other.suit != trump_suit {
                    Some(std::cmp::Ordering::Greater)
                } else if self.suit != trump_suit && other.suit == trump_suit {
                    Some(std::cmp::Ordering::Less)
                } else {
                    self.rank.partial_cmp(&other.rank)
                }
            }
        }
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
