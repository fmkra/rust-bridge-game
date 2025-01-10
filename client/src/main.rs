mod gui_client;
mod gui_create_room;
mod gui_lobby;
mod gui_login;
mod gui_play;
mod gui_room;
mod notifications;
mod utils;

use gui_client::{GuiClient, GuiClientState};
use gui_create_room::create_room_ui;
use gui_lobby::list_rooms;
use gui_login::login_ui;
use gui_play::{play_ui, preload_cards, preload_textures};
use gui_room::room_ui;

use common::{
    message::{
        client_message::{GetCardsMessage, JoinRoomMessage, ListPlacesMessage, ListRoomsMessage},
        server_notification::{
            AskBidNotification, AskTrickNotification, AuctionFinishedNotification,
            DummyCardsNotification, GameFinishedNotification, GameStartedNotification,
            JoinRoomNotification, LeaveRoomNotification, MakeBidNotification,
            SelectPlaceNotification, TrickFinishedNotification,
        },
        server_response::{
            GetCardsResponse, JoinRoomResponse, LeaveRoomResponse, ListPlacesResponse,
            ListRoomsResponse, LoginResponse, MakeBidResponse, MakeTrickResponse,
            RegisterRoomResponse, SelectPlaceResponse,
        },
        MessageTrait,
    },
    room::RoomId,
    Bid, Card,
};
use futures_util::FutureExt;
use macroquad::prelude::*;
use notifications::Notifier;
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde_json::to_string;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::{runtime::Runtime, time::sleep};
use utils::update_user_seat;

macro_rules! add_handler {
    ($builder:expr, $object:ty, $client_capture:expr, $notifier_capture:expr, |$client:ident, $notifier:ident, $msg:ident, $socket:ident| $body:block) => {
        $builder = $builder.on(<$object>::MSG_TYPE, {
            let $client = $client_capture.clone();
            let $notifier = $notifier_capture.clone();
            move |payload, $socket| {
                let $client = $client.clone();
                let $notifier = $notifier.clone();
                async move {
                    let $msg: $object = match payload {
                        Payload::Text(text) => serde_json::from_value(text[0].clone()).unwrap(),
                        _ => return,
                    };
                    $body
                }
                .boxed()
            }
        })
    };
}

#[macroquad::main("Bridge card game")]
async fn main() {
    let bid_textures = preload_textures().await;
    let card_textures = preload_cards().await;
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");

    let client = Arc::new(Mutex::new(GuiClient::new()));
    let notifier = Notifier::new();

    let socket = runtime.block_on(async {
        let mut builder = ClientBuilder::new("http://localhost:3000/").namespace("/");

        add_handler!(
            builder,
            LoginResponse,
            client,
            notifier,
            |client, notifier, msg, s| {
                match msg {
                    LoginResponse::Ok => {
                        println!("Login successful!");
                        {
                            let mut client_lock = client.lock().await;
                            client_lock.state = GuiClientState::InLobby;
                        }

                        s.emit(
                            ListRoomsMessage::MSG_TYPE,
                            to_string(&ListRoomsMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    LoginResponse::UsernameAlreadyExists => {
                        notifier.create_error(String::from("Username already exists"));
                    }
                    LoginResponse::UserAlreadyLoggedIn => {
                        notifier.create_error(String::from("User is already logged in"));
                    }
                }
            }
        );

        add_handler!(
            builder,
            ListRoomsResponse,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                let rooms = msg
                    .rooms
                    .iter()
                    .map(|room| room.as_str().to_string())
                    .collect();
                let mut client_lock = client.lock().await;
                client_lock.rooms = rooms;
            }
        );

        add_handler!(
            builder,
            RegisterRoomResponse,
            client,
            notifier,
            |client, _notifier, _msg, s| {
                let room_id = client.lock().await.selected_room_name.clone();
                s.emit(
                    JoinRoomMessage::MSG_TYPE,
                    to_string(&JoinRoomMessage {
                        room_id: RoomId::new(&room_id),
                    })
                    .unwrap(),
                )
                .await
                .unwrap();
            }
        );

        add_handler!(
            builder,
            JoinRoomResponse,
            client,
            notifier,
            |client, notifier, msg, s| {
                match msg {
                    JoinRoomResponse::Ok => {
                        {
                            let mut client_lock = client.lock().await;
                            client_lock.state = GuiClientState::InRoom;
                        }
                        s.emit(
                            ListPlacesMessage::MSG_TYPE,
                            to_string(&ListPlacesMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    JoinRoomResponse::Unauthenticated => {
                        notifier.create_error(String::from("You are not authenticated"));
                    }
                    JoinRoomResponse::AlreadyInRoom => {
                        notifier.create_error(String::from("You are already in the room"));
                    }
                    JoinRoomResponse::RoomNotFound => {
                        notifier.create_error(String::from("Room not found"));
                    }
                }
            }
        );

        add_handler!(
            builder,
            ListPlacesResponse,
            client,
            notifier,
            |client, notifier, msg, _s| {
                match msg {
                    ListPlacesResponse::Ok(msg) => {
                        let mut client_lock = client.lock().await;
                        client_lock.seats = msg;
                    }
                    ListPlacesResponse::NotInRoom => {
                        notifier.create_error(String::from("You are not in a room"));
                    }
                    ListPlacesResponse::Unauthenticated => {
                        notifier.create_error(String::from("You are not authenticated"));
                    }
                }
            }
        );

        add_handler!(
            builder,
            SelectPlaceResponse,
            client,
            notifier,
            |_client, notifier, msg, s| {
                match msg {
                    SelectPlaceResponse::Ok => {
                        s.emit(
                            ListPlacesMessage::MSG_TYPE,
                            to_string(&ListPlacesMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    SelectPlaceResponse::NotInRoom => {
                        notifier.create_error(String::from("You are not in a room"));
                    }
                    SelectPlaceResponse::PlaceAlreadyTaken => {
                        notifier.create_error(String::from("Place is already taken"));
                    }
                    SelectPlaceResponse::Unauthenticated => {
                        notifier.create_error(String::from("You are not authenticated"));
                    }
                }
            }
        );

        add_handler!(
            builder,
            SelectPlaceNotification,
            client,
            notifier,
            |client, notifier, msg, _s| {
                {
                    let mut client_lock = client.lock().await;
                    update_user_seat(&mut client_lock.seats, msg.user.clone(), msg.position);
                }
                let position_str = match msg.position {
                    Some(val) => format!("{}", val),
                    None => String::from("Spectator"),
                };
                notifier.create_info(String::from(&format!(
                    "Player {} selected position: {}",
                    msg.user.get_username(),
                    position_str
                )));
            }
        );

        add_handler!(
            builder,
            JoinRoomNotification,
            client,
            notifier,
            |_client, notifier, msg, _s| {
                notifier.create_info(String::from(&format!(
                    "Player {} joined the room.",
                    msg.user.get_username()
                )));
            }
        );

        add_handler!(
            builder,
            LeaveRoomResponse,
            client,
            notifier,
            |client, _notifier, _msg, s| {
                {
                    let mut client_lock = client.lock().await;
                    client_lock.state = GuiClientState::InLobby;
                    client_lock.seats = [None, None, None, None];
                    client_lock.selected_seat = None;
                }
                s.emit(
                    ListRoomsMessage::MSG_TYPE,
                    to_string(&ListRoomsMessage {}).unwrap(),
                )
                .await
                .unwrap();
            }
        );

        add_handler!(
            builder,
            LeaveRoomNotification,
            client,
            notifier,
            |client, notifier, msg, _s| {
                let mut client_lock = client.lock().await;
                update_user_seat(&mut client_lock.seats, msg.user.clone(), None);
                notifier.create_info(String::from(&format!(
                    "Player {} left the room.",
                    msg.user.get_username()
                )));
            }
        );

        add_handler!(
            builder,
            GameStartedNotification,
            client,
            notifier,
            |client, notifier, _msg, s| {
                {
                    let mut client_lock = client.lock().await;
                    client_lock.state = GuiClientState::Playing;
                }
                notifier.create_info(String::from("Game started"));
                s.emit(
                    GetCardsMessage::MSG_TYPE,
                    to_string(&GetCardsMessage {}).unwrap(),
                )
                .await
                .unwrap();
            }
        );

        add_handler!(
            builder,
            GetCardsResponse,
            client,
            notifier,
            |client, notifier, msg, _s| {
                match msg {
                    GetCardsResponse::Ok { cards, position } => {
                        let mut client_lock = client.lock().await;
                        client_lock.card_list = Some(cards);
                        client_lock.selected_seat = Some(position);
                    }
                    GetCardsResponse::NotInRoom => {
                        notifier.create_error(String::from("You are not in a room"));
                    }
                    GetCardsResponse::Unauthenticated => {
                        notifier.create_error(String::from("You are not authenticated"));
                    }
                    GetCardsResponse::SpectatorNotAllowed => {
                        notifier.create_error(String::from("Spectator is not allowed to play"));
                    }
                }
            }
        );

        add_handler!(
            builder,
            MakeBidNotification,
            client,
            notifier,
            |client, notifier, msg, _s| {
                let mut client_lock = client.lock().await;
                client_lock.player_bids[msg.player.to_usize()] = Some(msg.bid);
                notifier.create_info(String::from(&format!(
                    "Player {} bid {}",
                    msg.player, msg.bid
                )));
            }
        );

        add_handler!(
            builder,
            AskBidNotification,
            client,
            notifier,
            |client, notifier, msg, _s| {
                let bid_message = match msg.max_bid {
                    Bid::Pass => String::from("There is no max bid"),
                    _ => String::from(format!("Current max bid is {}", msg.max_bid)),
                };
                let mut client_lock = client.lock().await;
                client_lock.game_current_player = Some(msg.player);

                notifier.create_info(bid_message);
                notifier.create_info(String::from(&format!(
                    "Player {} is bidding right now.",
                    msg.player
                )));
            }
        );

        add_handler!(
            builder,
            MakeBidResponse,
            client,
            notifier,
            |_client, notifier, msg, _s| {
                match msg {
                    MakeBidResponse::Ok => {}
                    MakeBidResponse::AuctionNotInProcess => {
                        notifier.create_error(String::from("Auction is not in process"));
                    }
                    MakeBidResponse::NotInRoom => {
                        notifier.create_error(String::from("You are not in a room"));
                    }
                    MakeBidResponse::Unauthenticated => {
                        notifier.create_error(String::from("You are not authenticated"));
                    }
                    MakeBidResponse::SpectatorNotAllowed => {
                        notifier.create_error(String::from("You are not allowed to play"));
                    }
                    MakeBidResponse::NotYourTurn => {
                        notifier.create_error(String::from("It's not your turn"));
                    }
                    MakeBidResponse::InvalidBid => {
                        notifier.create_error(String::from("This bid is not valid"))
                    }
                }
            }
        );

        add_handler!(
            builder,
            AuctionFinishedNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                let msg = match msg {
                    AuctionFinishedNotification::Winner(msg) => msg,
                    AuctionFinishedNotification::NoWinner => panic!("No winner in auction"),
                };
                {
                    let mut client_lock = client.lock().await;
                    client_lock.game_max_bid = Some(msg.max_bid);
                    client_lock.game_current_player = Some(msg.winner);
                }
            }
        );

        add_handler!(
            builder,
            DummyCardsNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                {
                    let mut client_lock = client.lock().await;
                    client_lock.dummy_cards = Some(msg.cards);
                    client_lock.dummy_player = Some(msg.dummy);
                }
            }
        );

        add_handler!(
            builder,
            AskTrickNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                let mut client_lock = client.lock().await;

                if let Some(dummy_cards) = client_lock.dummy_cards.as_mut() {
                    if let Some(card) = msg.cards.last() {
                        dummy_cards.retain(|c| c != card);
                    }
                }

                client_lock.game_current_player = Some(msg.player);

                let mut placed_cards: [Option<Card>; 4] = [None, None, None, None];
                let mut previous_player = msg.player.prev();
                for el in msg.cards.iter().rev() {
                    placed_cards[previous_player.to_usize()] = Some(el.clone());
                    previous_player = previous_player.prev();
                }
                client_lock.current_placed_cards = placed_cards;
            }
        );

        add_handler!(
            builder,
            MakeTrickResponse,
            client,
            notifier,
            |client, notifier, msg, _s| {
                match msg {
                    MakeTrickResponse::Ok => {
                        let mut client_lock = client.lock().await;

                        let placed_trick = client_lock.placed_trick.clone();
                        if let Some(cards) = client_lock.card_list.as_mut() {
                            if let Some(placed_card) = placed_trick {
                                cards.retain(|c| *c != placed_card);
                            }
                        }
                    }
                    MakeTrickResponse::NotInRoom => {
                        notifier.create_error(String::from("You are not in a room"));
                    }
                    MakeTrickResponse::SpectatorNotAllowed => {
                        notifier.create_error(String::from("You are not allowed to play"));
                    }
                    MakeTrickResponse::NotYourTurn => {
                        notifier.create_error(String::from("It's not your turn"));
                    }
                    MakeTrickResponse::TrickNotInProcess => {
                        notifier.create_error(String::from("Trick is not in process"));
                    }
                    MakeTrickResponse::InvalidCard => {
                        notifier.create_error(String::from("This card is not valid"));
                    }
                    MakeTrickResponse::Unauthenticated => {
                        notifier.create_error(String::from("You are not authenticated"));
                    }
                }
            }
        );

        add_handler!(
            builder,
            TrickFinishedNotification,
            client,
            notifier,
            |client, notifier, msg, _s| {
                {
                    let mut client_lock = client.lock().await;

                    if let Some(dummy_cards) = client_lock.dummy_cards.as_mut() {
                        if let Some(card) = msg.cards.last() {
                            dummy_cards.retain(|c| c != card);
                        }
                    }

                    if let Some(current_player) = client_lock.game_current_player {
                        let placed_cards = client_lock.current_placed_cards.as_mut();
                        if let Some(last_tricked) = msg.cards.last() {
                            placed_cards[current_player.to_usize()] = Some(*last_tricked);
                        }
                    }
                }
                notifier.create_info(String::from(format!(
                    "Trick {} taken by {:?}",
                    msg.cards
                        .iter()
                        .map(Card::to_string)
                        .collect::<Vec<_>>()
                        .join(" "),
                    msg.taker
                )));
            }
        );

        add_handler!(
            builder,
            GameFinishedNotification,
            client,
            notifier,
            |client, notifier, _msg, s| {
                notifier.create_info(String::from("Game finished!"));
                sleep(Duration::from_secs(5)).await;
                {
                    let mut client_lock = client.lock().await;
                    client_lock.state = GuiClientState::InLobby;
                    client_lock.selected_room_name = String::new();
                    client_lock.seats = [None, None, None, None];
                    client_lock.selected_seat = None;
                    client_lock.card_list = None;
                    client_lock.player_bids = [None, None, None, None];
                    client_lock.placed_bid = None;
                    client_lock.placed_trick = None;
                    client_lock.game_max_bid = None;
                    client_lock.game_current_player = None;
                    client_lock.dummy_cards = None;
                    client_lock.dummy_player = None;
                    client_lock.current_placed_cards = [None, None, None, None];
                }
                s.emit(
                    ListRoomsMessage::MSG_TYPE,
                    to_string(&ListRoomsMessage {}).unwrap(),
                )
                .await
                .unwrap();
            }
        );

        Arc::new(builder.connect().await.expect("Connection failed"))
    });

    loop {
        clear_background(Color::from_rgba(50, 115, 85, 255));

        let mut client_lock = client.blocking_lock();
        let current_state = client_lock.state;

        match current_state {
            GuiClientState::Logging => {
                login_ui(socket.clone(), &runtime, &mut client_lock.name);
            }
            GuiClientState::InLobby => {
                list_rooms(socket.clone(), &runtime, &mut client_lock);
            }
            GuiClientState::CreatingRoom => {
                create_room_ui(socket.clone(), &runtime, &mut client_lock);
            }
            GuiClientState::InRoom => {
                room_ui(socket.clone(), &runtime, &mut client_lock);
            }
            GuiClientState::Playing => {
                play_ui(
                    socket.clone(),
                    &runtime,
                    &mut client_lock,
                    &bid_textures,
                    &card_textures,
                );
            }
        }

        notifier.display().await;

        next_frame().await;
    }
}
