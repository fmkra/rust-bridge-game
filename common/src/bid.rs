use serde::{Deserialize, Serialize};

use crate::card::Suit;
use std::{cmp::Ordering, fmt};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BidType {
    Trump(Suit),
    NoTrump,
}

impl BidType {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Trump(suit) => suit.to_str(),
            Self::NoTrump => "No Trump",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bid {
    Pass,
    Play(u8, BidType),
    Double,
    Redouble,
}

impl Bid {
    pub fn new(number: u8, typ: BidType) -> Option<Bid> {
        if (1..=7).contains(&number) {
            Some(Bid::Play(number, typ))
        } else {
            None
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Play(_, _) => 1,
            Self::Double => 2,
            Self::Redouble => 3,
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            Self::Pass => "Pass".into(),
            Self::Play(number, typ) => format!("{} {}", number, typ.to_str()),
            Self::Double => "Double".into(),
            Self::Redouble => "Redouble".into(),
        }
    }
}

impl fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bid::Pass => write!(f, "Pass"),
            Bid::Double => write!(f, "Double"),
            Bid::Redouble => write!(f, "Redouble"),
            Bid::Play(number, BidType::Trump(suit)) => write!(f, "{} of {}", number, suit.to_str()),
            Bid::Play(number, BidType::NoTrump) => write!(f, "{} No Trump", number),
        }
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_u8 = self.to_u8();
        let other_u8 = other.to_u8();
        if self_u8 < other_u8 {
            Ordering::Less
        } else if self_u8 == other_u8 {
            match (self, other) {
                (Bid::Play(self_number, self_type), Bid::Play(other_number, other_type)) => {
                    match self_number.cmp(other_number) {
                        Ordering::Equal => self_type.cmp(other_type),
                        other => other,
                    }
                }
                _ => Ordering::Equal,
            }
        } else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryInto<BidType> for Bid {
    type Error = ();

    fn try_into(self) -> Result<BidType, Self::Error> {
        match self {
            Bid::Play(_, typ) => Ok(typ),
            _ => Err(()),
        }
    }
}
