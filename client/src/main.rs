mod client;
mod gui;
mod notifications;
mod utils;

use client::{Client, ClientState};
use common::message::server_notification::DealFinishedNotification;
use gui::create_room::create_room_ui;
use gui::lobby::list_rooms;
use gui::login::login_ui;
use gui::play::{play_ui, preload_cards, preload_textures};
use gui::room::room_ui;

use common::{
    message::{
        client_message::{GetCardsMessage, JoinRoomMessage, ListPlacesMessage, ListRoomsMessage},
        server_notification::{
            AskBidNotification, AskTrickNotification, AuctionFinishedNotification,
            DummyCardsNotification, GameFinishedNotification, GameStartedNotification,
            JoinRoomNotification, LeaveRoomNotification, MakeBidNotification,
            MakeTrickNotification, SelectPlaceNotification, TrickFinishedNotification,
        },
        server_response::{
            GetCardsResponse, JoinRoomResponse, LeaveRoomResponse, ListPlacesResponse,
            ListRoomsResponse, LoginResponse, MakeBidResponse, MakeTrickResponse,
            RegisterRoomResponse, SelectPlaceResponse,
        },
        GetErrorMessage, MessageTrait,
    },
    room::RoomId,
    Card,
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

    let client = Arc::new(Mutex::new(Client::new()));
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
                        client.lock().await.state = ClientState::InLobby;

                        s.emit(
                            ListRoomsMessage::MSG_TYPE,
                            to_string(&ListRoomsMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    err => notifier.create_error(err.err_msg()),
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

                client.lock().await.rooms = rooms;
            }
        );

        add_handler!(
            builder,
            RegisterRoomResponse,
            client,
            notifier,
            |client, _notifier, _msg, s| {
                let room_id = RoomId::new(client.lock().await.selected_room_name.clone().into());
                s.emit(
                    JoinRoomMessage::MSG_TYPE,
                    to_string(&JoinRoomMessage { room_id }).unwrap(),
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
                        client.lock().await.state = ClientState::InRoom;

                        s.emit(
                            ListPlacesMessage::MSG_TYPE,
                            to_string(&ListPlacesMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    err => notifier.create_error(err.err_msg()),
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
                    err => notifier.create_error(err.err_msg()),
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
                    err => notifier.create_error(err.err_msg()),
                }
            }
        );

        add_handler!(
            builder,
            SelectPlaceNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                update_user_seat(
                    &mut client.lock().await.seats,
                    msg.user.clone(),
                    msg.position,
                );
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
                    client_lock.state = ClientState::InLobby;
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
                update_user_seat(&mut client.lock().await.seats, msg.user.clone(), None);

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
            |client, _notifier, _msg, s| {
                client.lock().await.state = ClientState::Playing;

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
                    err => notifier.create_error(err.err_msg()),
                }
            }
        );

        add_handler!(
            builder,
            AskBidNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                let mut client_lock = client.lock().await;
                client_lock.game_current_player = Some(msg.player);
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
                    err => notifier.create_error(err.err_msg()),
                }
            }
        );

        add_handler!(
            builder,
            MakeBidNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                let mut client_lock = client.lock().await;
                client_lock.player_bids[msg.player.to_usize()] = Some(msg.bid);
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
                let mut client_lock = client.lock().await;
                client_lock.game_max_bid = Some(msg.max_bid);
                client_lock.game_max_bidder = Some(msg.winner);
                client_lock.game_current_player = Some(msg.winner);
                client_lock.player_bids = [None, None, None, None];
            }
        );

        add_handler!(
            builder,
            DummyCardsNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                let mut client_lock = client.lock().await;
                client_lock.dummy_cards = Some(msg.cards);
                client_lock.dummy_player = Some(msg.dummy);
            }
        );

        add_handler!(
            builder,
            AskTrickNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                client.lock().await.game_current_player = Some(msg.player);
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
                    err => notifier.create_error(err.err_msg()),
                }
            }
        );

        add_handler!(
            builder,
            MakeTrickNotification,
            client,
            notifier,
            |client, _notifier, msg, _s| {
                let mut client_lock = client.lock().await;

                // Show card on the table
                client_lock.current_placed_cards[msg.player.to_usize()] = Some(msg.card);

                // Remove card from dummy cards
                if let Some(dummy_cards) = client_lock.dummy_cards.as_mut() {
                    dummy_cards.retain(|c| *c != msg.card);
                }

                // Remove card from my hand
                if let Some(cards) = client_lock.card_list.as_mut() {
                    cards.retain(|c| *c != msg.card);
                }
            }
        );

        add_handler!(
            builder,
            TrickFinishedNotification,
            client,
            notifier,
            |client, notifier, msg, _s| {
                client.lock().await.current_placed_cards = [None, None, None, None];

                // TODO: show taken by
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
            DealFinishedNotification,
            client,
            notifier,
            |client, notifier, msg, s| {
                if msg.contract_succeeded {
                    notifier.create_info(format!("Contract won by {}", msg.bidder));
                } else {
                    notifier.create_info(format!("Contract lost by {}", msg.bidder));
                }
                let mut client_lock = client.lock().await;
                client_lock.points = msg.points;
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
            GameFinishedNotification,
            client,
            notifier,
            |client, notifier, _msg, s| {
                notifier.create_info(String::from("Game finished!"));
                sleep(Duration::from_secs(5)).await;
                {
                    let mut client_lock = client.lock().await;
                    client_lock.state = ClientState::InLobby;
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
            ClientState::Logging => {
                login_ui(socket.clone(), &runtime, &mut client_lock.name);
            }
            ClientState::InLobby => {
                list_rooms(socket.clone(), &runtime, &mut client_lock);
            }
            ClientState::CreatingRoom => {
                create_room_ui(socket.clone(), &runtime, &mut client_lock);
            }
            ClientState::InRoom => {
                room_ui(socket.clone(), &runtime, &mut client_lock);
            }
            ClientState::Playing => {
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
