use std::{
    collections::VecDeque, io::Write, sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }
};

use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::runtime::Runtime;

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

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum GuiClientState {
    Logging,
    InLobby,
    CreatingRoom,
    InRoom,
    InSeat,
    Playing,
}

pub struct GuiClient {
    pub name: Arc<Mutex<Option<String>>>,
    pub state: Arc<Mutex<GuiClientState>>,
    pub errors: Arc<Mutex<VecDeque<String>>>,
    pub rooms: Arc<Mutex<Vec<String>>>,
    pub room_name: Arc<Mutex<Option<String>>>,

    // selected_card: Arc<Mutex<Option<Card>>>,
    // selected_card_clone: Arc<Mutex<Option<Card>>>,
    // card_list: Arc<Mutex<Option<Vec<Card>>>>,
    // card_list_clone: Arc<Mutex<Option<Vec<Card>>>>,
    // card_list_clone_2: Arc<Mutex<Option<Vec<Card>>>>,
    // card_list_notify: Arc<Notify>,
    // card_list_notify_clone: Arc<Notify>,

    // auction_result: Arc<Mutex<AuctionFinishedNotification>>,
    // auction_result_clone: Arc<Mutex<AuctionFinishedNotification>>,

    // register_room_notifier: Arc<Notify>,
    // register_room_notifier_clone: Arc<Notify>,
    // game_finished: Arc<AtomicBool>,
    // game_finished_clone: Arc<AtomicBool>,
}

impl GuiClient {
    pub fn new() -> GuiClient {
        GuiClient {
            name: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(GuiClientState::Logging)),
            errors: Arc::new(Mutex::new(VecDeque::new())),
            rooms: Arc::new(Mutex::new(Vec::new())),
            room_name: Arc::new(Mutex::new(None)),
        }
    }
}