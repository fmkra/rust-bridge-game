use core::cmp::Ordering;

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

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BidType {
    Trump(Suit),
    NoTrump,
}

// It is guaranteed that number \in [1 ; 7]
#[derive(Debug, PartialEq, Eq)]
pub struct Bid {
    pub number: u8,
    pub typ: BidType,
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

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.number.cmp(&other.number) {
            Ordering::Equal => self.typ.cmp(&other.typ),
            other => other,
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

    pub fn compare_with_trump(
        &self,
        other: &Card,
        bid_type: &BidType
    ) -> Option<Ordering> {
        match bid_type {
            BidType::NoTrump => {
                self.partial_cmp(&other)
            },
            BidType::Trump(trump_suit) => {
                if self.suit == *trump_suit && other.suit != *trump_suit {
                    Some(Ordering::Greater)
                } else if self.suit != *trump_suit && other.suit == *trump_suit {
                    Some(Ordering::Less)
                } else {
                    self.partial_cmp(&other)
                }
            }
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

pub struct Game {
    pub current_bid: Bid,
    pub current_trick: Vec<Card>
}

impl Game {
    pub fn new(bid: Bid) -> Game {
        Game {
            current_bid: bid,
            current_trick: Vec::new(),
        }
    }

    pub fn add_card(&mut self, c1: Card) {
        self.current_trick.push(c1);
    }

    pub fn trick_max(&self) -> Option<&Card>{
        let mut cur_max = None;
        for el in &self.current_trick {
            println!("Before: {:?}", cur_max);
            cur_max = match cur_max {
                None => Some(el),
                Some(card) => {
                    match card.compare_with_trump(el, &self.current_bid.typ) {
                        Some(Ordering::Less) => Some(el),
                        _ => Some(card),
                    }
                }
            };
            println!("After: {:?}", cur_max);
        }
        return cur_max;
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}