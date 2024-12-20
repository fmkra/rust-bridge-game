use serde::{Deserialize, Serialize};

use crate::card::Suit;
use std::cmp::Ordering;

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
}

impl Bid {
    pub fn new(number: u8, typ: BidType) -> Option<Bid> {
        if number >= 1 && number <= 7 {
            Some(Bid::Play(number, typ))
        } else {
            None
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            Self::Pass => "Pass".into(),
            Self::Play(number, typ) => format!("{} {}", number, typ.to_str()),
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

impl TryInto<BidType> for Bid {
    type Error = ();

    fn try_into(self) -> Result<BidType, Self::Error> {
        match self {
            Bid::Pass => Err(()),
            Bid::Play(_, typ) => Ok(typ),
        }
    }
}
