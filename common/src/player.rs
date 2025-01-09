use std::convert::TryInto;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Deserialize, Serialize)]
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

    pub fn prev(&self) -> Player {
        self.skip(3)
    }

    pub fn skip(&self, num_skips: usize) -> Player {
        // Unwrap() is valid, as the number is in [0; 3]
        Player::from_usize((self.to_usize() + num_skips) % 4).unwrap()
    }

    pub fn get_partner(&self) -> Player {
        self.skip(2)
    }

    pub fn is_opponent(&self, player: Player) -> bool {
        player == self.skip(1) || player == self.skip(3)
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

    pub fn to_str(&self) -> &str {
        match self {
            Player::North => "North",
            Player::East => "East",
            Player::South => "South",
            Player::West => "West",
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
