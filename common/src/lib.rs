pub mod bid;
pub mod card;
pub mod game;
pub mod player;

pub use bid::{Bid, BidType};
pub use card::{Card, Rank, Suit};
pub use game::{
    BidStatus, 
    BidError,
    TrickStatus, 
    TrickError,
    TrickState, 
    Game, 
    GameState
};
pub use player::Player;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
