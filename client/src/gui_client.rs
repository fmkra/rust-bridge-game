use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use common::{user::User, Bid, Card, Player};

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum GuiClientState {
    Logging,
    InLobby,
    CreatingRoom,
    InRoom,
    Playing,
}

pub struct GuiClient {
    pub name: String,
    pub state: GuiClientState,
    pub rooms: Vec<String>,
    pub selected_room_name: Option<String>,
    pub seats: [Option<User>; 4],
    pub selected_seat: Option<Player>,
    pub card_list: Option<Vec<Card>>,
    pub player_bids: [Option<Bid>; 4], // TODO: Display placed bids in the ui.
    pub placed_bid: Option<Bid>,
    pub placed_trick: Option<Card>,
    pub game_max_bid: Option<Bid>,
    pub game_current_player: Option<Player>,
    pub dummy_cards: Option<Vec<Card>>,
    pub dummy_player: Option<Player>,
    pub current_placed_cards: [Option<Card>; 4],
}

impl GuiClient {
    pub fn new() -> GuiClient {
        GuiClient {
            name: String::new(),
            state: GuiClientState::Logging,
            rooms: Vec::new(),
            selected_room_name: None,
            seats: [None, None, None, None],
            selected_seat: None,
            card_list: None,
            player_bids: [None, None, None, None],
            placed_bid: None,
            placed_trick: None,
            game_max_bid: None,
            game_current_player: None,
            dummy_cards: None,
            dummy_player: None,
            current_placed_cards: [None, None, None, None],
        }
    }
}
