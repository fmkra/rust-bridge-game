use serde::{Deserialize, Serialize};

use common::{user::User, Bid, Card, Player};

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClientState {
    Logging,
    InLobby,
    CreatingRoom,
    InRoom,
    Playing,
}

pub struct Client {
    pub name: String,
    pub state: ClientState,
    pub rooms: Vec<String>,
    pub selected_room_name: String,
    pub seats: [Option<User>; 4],
    pub selected_seat: Option<Player>,
    pub card_list: Option<Vec<Card>>,
    pub player_bids: [Option<Bid>; 4],
    pub placed_bid: Option<Bid>,
    pub placed_trick: Option<Card>,
    pub game_max_bid: Option<Bid>,
    pub game_max_bidder: Option<Player>,
    pub game_current_player: Option<Player>,
    pub dummy_cards: Option<Vec<Card>>,
    pub dummy_player: Option<Player>,
    pub current_placed_cards: [Option<Card>; 4],
}

impl Client {
    pub fn new() -> Client {
        Client {
            name: String::new(),
            state: ClientState::Logging,
            rooms: Vec::new(),
            selected_room_name: String::new(),
            seats: [None, None, None, None],
            selected_seat: None,
            card_list: None,
            player_bids: [None, None, None, None],
            placed_bid: None,
            placed_trick: None,
            game_max_bid: None,
            game_max_bidder: None,
            game_current_player: None,
            dummy_cards: None,
            dummy_player: None,
            current_placed_cards: [None, None, None, None],
        }
    }
}
