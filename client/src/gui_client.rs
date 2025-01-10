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
    pub name: Arc<Mutex<Option<String>>>,
    pub state: Arc<Mutex<GuiClientState>>,
    pub rooms: Arc<Mutex<Vec<String>>>,
    pub selected_room_name: Arc<Mutex<Option<String>>>,
    pub seats: Arc<Mutex<[Option<User>; 4]>>,
    pub selected_seat: Arc<Mutex<Option<Player>>>,
    pub card_list: Arc<Mutex<Option<Vec<Card>>>>,
    pub player_bids: Arc<Mutex<[Option<Bid>; 4]>>, // TODO: Display placed bids in the ui.
    pub placed_bid: Arc<Mutex<Option<Bid>>>,
    pub placed_trick: Arc<Mutex<Option<Card>>>,
    pub game_max_bid: Arc<Mutex<Option<Bid>>>,
    pub game_current_player: Arc<Mutex<Option<Player>>>,
    pub dummy_cards: Arc<Mutex<Option<Vec<Card>>>>,
    pub dummy_player: Arc<Mutex<Option<Player>>>,
    pub current_placed_cards: Arc<Mutex<[Option<Card>; 4]>>,
}

impl GuiClient {
    pub fn new() -> GuiClient {
        GuiClient {
            name: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(GuiClientState::Logging)),
            rooms: Arc::new(Mutex::new(Vec::new())),
            selected_room_name: Arc::new(Mutex::new(None)),
            seats: Arc::new(Mutex::new([None, None, None, None])),
            selected_seat: Arc::new(Mutex::new(None)),
            card_list: Arc::new(Mutex::new(None)),
            player_bids: Arc::new(Mutex::new([None, None, None, None])),
            placed_bid: Arc::new(Mutex::new(None)),
            placed_trick: Arc::new(Mutex::new(None)),
            game_max_bid: Arc::new(Mutex::new(None)),
            game_current_player: Arc::new(Mutex::new(None)),
            dummy_cards: Arc::new(Mutex::new(None)),
            dummy_player: Arc::new(Mutex::new(None)),
            current_placed_cards: Arc::new(Mutex::new([None, None, None, None])),
        }
    }
}
