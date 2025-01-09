use std::{
    collections::VecDeque,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex, Notify};

use common::{
    message::{
        client_message::{
            GetCardsMessage, JoinRoomMessage, LeaveRoomMessage, ListPlacesMessage,
            ListRoomsMessage, LoginMessage, MakeBidMessage, MakeTrickMessage, RegisterRoomMessage,
            SelectPlaceMessage, GET_CARDS_MESSAGE, JOIN_ROOM_MESSAGE, LEAVE_ROOM_MESSAGE,
            LIST_PLACES_MESSAGE, LIST_ROOMS_MESSAGE, LOGIN_MESSAGE, MAKE_BID_MESSAGE,
            MAKE_TRICK_MESSAGE, REGISTER_ROOM_MESSAGE, SELECT_PLACE_MESSAGE,
        },
        server_notification::{
            AskBidNotification, AskTrickNotification, AuctionFinishedNotification,
            DummyCardsNotification, GameFinishedNotification, GameStartedNotification,
            JoinRoomNotification, LeaveRoomNotification, SelectPlaceNotification,
            TrickFinishedNotification, ASK_BID_NOTIFICATION, ASK_TRICK_NOTIFICATION,
            AUCTION_FINISHED_NOTIFICATION, DUMMY_CARDS_NOTIFICATION, GAME_FINISHED_NOTIFICATION,
            GAME_STARTED_NOTIFICATION, JOIN_ROOM_NOTIFICATION, LEAVE_ROOM_NOTIFICATION,
            SELECT_PLACE_NOTIFICATION, TRICK_FINISHED_NOTIFICATION,
        },
        server_response::{
            GetCardsResponse, LeaveRoomResponse, ListPlacesResponse, ListRoomsResponse,
            LoginResponse, MakeBidResponse, MakeTrickResponse, SelectPlaceResponse,
            GET_CARDS_RESPONSE, JOIN_ROOM_RESPONSE, LEAVE_ROOM_RESPONSE, LIST_PLACES_RESPONSE,
            LIST_ROOMS_RESPONSE, LOGIN_RESPONSE, MAKE_BID_RESPONSE, MAKE_TRICK_RESPONSE,
            REGISTER_ROOM_RESPONSE, SELECT_PLACE_RESPONSE,
        },
    },
    room::{RoomId, RoomInfo, Visibility},
    user::User,
    Bid, BidType, Card, Player, Rank, Suit,
};

use crate::gui_notification::Notification;

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
    pub notifications: Arc<Mutex<VecDeque<Notification>>>,
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
            notifications: Arc::new(Mutex::new(VecDeque::new())),
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
