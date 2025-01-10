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
            JoinRoomNotification, LeaveRoomNotification, SelectPlaceNotification,
            TrickFinishedNotification,
        },
        server_response::{
            GetCardsResponse, JoinRoomResponse, LeaveRoomResponse, ListPlacesResponse,
            ListRoomsResponse, LoginResponse, MakeBidResponse, MakeTrickResponse,
            RegisterRoomResponse, SelectPlaceResponse,
        },
        MessageTrait,
    },
    room::RoomId,
    Bid, Card, Player,
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

#[macroquad::main("Bridge card game")]
async fn main() {
    let bid_textures = preload_textures().await;
    let card_textures = preload_cards().await;
    let client = Arc::new(Mutex::new(GuiClient::new()));
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    // Clones of Arcs used in handling gui inputs
    let input_nickname = Arc::new(Mutex::new(String::new()));
    let input_nickname_clone = input_nickname.clone();
    let input_created_room_name = Arc::new(Mutex::new(String::new()));
    let input_created_room_name_clone = input_created_room_name.clone();
    let input_selected_room_name: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let input_selected_room_name_clone: Arc<Mutex<Option<String>>> =
        input_selected_room_name.clone();
    let input_selected_room_name_clone_1 = input_selected_room_name_clone.clone();
    let input_selected_seat: Arc<Mutex<Option<Player>>> = Arc::new(Mutex::new(None));
    let input_selected_seat_clone = input_selected_seat.clone();
    let input_placed_bid: Arc<Mutex<Option<Bid>>> = Arc::new(Mutex::new(None));
    let input_placed_bid_clone = input_placed_bid.clone();
    let input_placed_bid_clone_1 = input_placed_bid.clone();
    // let input_selected_bid =

    // Clones of GuiClient Arc fields
    let notifier = Notifier::new();
    let notifier_clone_0 = notifier.clone();
    let notifier_clone_1 = notifier.clone();
    let notifier_clone_2 = notifier.clone();
    let notifier_clone_3 = notifier.clone();
    let notifier_clone_4 = notifier.clone();
    let notifier_clone_5 = notifier.clone();
    let notifier_clone_6 = notifier.clone();
    let notifier_clone_7 = notifier.clone();
    let notifier_clone_8 = notifier.clone();
    let notifier_clone_9 = notifier.clone();
    let notifier_clone_10 = notifier.clone();
    let notifier_clone_11 = notifier.clone();
    let notifier_clone_12 = notifier.clone();
    let notifier_clone_13 = notifier.clone();

    let client_clone_0 = client.clone();
    let client_clone_1 = client.clone();
    let client_clone_2 = client.clone();
    let client_clone_3 = client.clone();
    let client_clone_4 = client.clone();
    let client_clone_5 = client.clone();
    let client_clone_6 = client.clone();
    let client_clone_7 = client.clone();
    let client_clone_8 = client.clone();
    let client_clone_9 = client.clone();
    let client_clone_10 = client.clone();
    let client_clone_11 = client.clone();
    let client_clone_12 = client.clone();
    let client_clone_13 = client.clone();
    let client_clone_14 = client.clone();
    let client_clone_15 = client.clone();
    let client_clone_16 = client.clone();
    let client_clone_17 = client.clone();

    // Connect to the server
    let socket = runtime.block_on(async {
        Arc::new(
            ClientBuilder::new("http://localhost:3000/")
                .namespace("/")
                .on(LoginResponse::MSG_TYPE, move |payload, c| {
                    let client = client_clone_0.clone();
                    let notifier = notifier_clone_0.clone();
                    let nickname = input_nickname_clone.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<LoginResponse>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };

                        match msg {
                            LoginResponse::Ok => {
                                println!("Login successful!");
                                {
                                    let mut client_lock = client.lock().await;

                                    let nickname_val = nickname.lock().await;
                                    client_lock.name = Some(nickname_val.clone());
                                    client_lock.state = GuiClientState::InLobby;
                                }

                                c.emit(
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
                    .boxed()
                })
                .on(ListRoomsResponse::MSG_TYPE, move |payload, _| {
                    let client = client_clone_1.clone();
                    async move {
                        let rooms = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<ListRoomsResponse>(text[0].clone())
                                    .unwrap()
                                    .rooms
                                    .iter()
                                    .map(|room| room.as_str().to_string())
                                    .collect()
                            }
                            _ => vec![],
                        };
                        let mut client_lock = client.lock().await;
                        client_lock.rooms = rooms;
                    }
                    .boxed()
                })
                .on(RegisterRoomResponse::MSG_TYPE, move |_, c| {
                    let input_selected_room_name_arc = input_selected_room_name_clone.clone();
                    let input_created_room_name_arc = input_created_room_name_clone.clone();
                    async move {
                        let room_id = {
                            let mut input_selected_room_name_val =
                                input_selected_room_name_arc.lock().await;
                            let input_created_room_name_val =
                                input_created_room_name_arc.lock().await;
                            *input_selected_room_name_val =
                                Some(input_created_room_name_val.clone());
                            input_created_room_name_val.clone()
                        };
                        c.emit(
                            JoinRoomMessage::MSG_TYPE,
                            to_string(&JoinRoomMessage {
                                room_id: RoomId::new(room_id.as_str()),
                            })
                            .unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    .boxed()
                })
                .on(JoinRoomResponse::MSG_TYPE, move |payload, c| {
                    let client = client_clone_2.clone();
                    let notifier = notifier_clone_2.clone();
                    let input_selected_room_name_arc = input_selected_room_name_clone_1.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<JoinRoomResponse>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        match msg {
                            JoinRoomResponse::Ok => {
                                {
                                    let mut client_lock = client.lock().await;
                                    client_lock.state = GuiClientState::InRoom;

                                    let input_selected_room_name_val =
                                        input_selected_room_name_arc.lock().await;
                                    client_lock.selected_room_name =
                                        input_selected_room_name_val.clone();
                                }
                                c.emit(
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
                    .boxed()
                })
                .on(ListPlacesResponse::MSG_TYPE, move |payload, _| {
                    let client = client_clone_3.clone();
                    let notifier = notifier_clone_1.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<ListPlacesResponse>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
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
                    .boxed()
                })
                .on(SelectPlaceResponse::MSG_TYPE, move |payload, c| {
                    let client = client_clone_4.clone();
                    let notifier = notifier_clone_3.clone();
                    let input_selected_seat_arc = input_selected_seat_clone.clone();
                    async move {
                        match payload {
                            Payload::Text(text) => {
                                let msg =
                                    serde_json::from_value::<SelectPlaceResponse>(text[0].clone())
                                        .unwrap();
                                match msg {
                                    SelectPlaceResponse::Ok => {
                                        {
                                            let mut client_lock = client.lock().await;
                                            let input_selected_seat_val =
                                                input_selected_seat_arc.lock().await;
                                            client_lock.selected_seat = *input_selected_seat_val;
                                        };
                                        // Refreshes the seats
                                        c.emit(
                                            ListPlacesMessage::MSG_TYPE,
                                            to_string(&ListPlacesMessage {}).unwrap(),
                                        )
                                        .await
                                        .unwrap();
                                    }
                                    SelectPlaceResponse::NotInRoom => {
                                        notifier
                                            .create_error(String::from("You are not in a room"));
                                    }
                                    SelectPlaceResponse::PlaceAlreadyTaken => {
                                        notifier
                                            .create_error(String::from("Place is already taken"));
                                    }
                                    SelectPlaceResponse::Unauthenticated => {
                                        notifier.create_error(String::from(
                                            "You are not authenticated",
                                        ));
                                    }
                                };
                            }
                            _ => return,
                        };
                    }
                    .boxed()
                })
                .on(SelectPlaceNotification::MSG_TYPE, move |payload, _| {
                    let client = client_clone_5.clone();
                    let notifier = notifier_clone_4.clone();
                    async move {
                        let player_position = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<SelectPlaceNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_lock = client.lock().await;
                            update_user_seat(
                                &mut client_lock.seats,
                                player_position.user.clone(),
                                player_position.position,
                            );
                        }
                        let position_str = match player_position.position {
                            Some(val) => format!("{}", val),
                            None => String::from("Spectator"),
                        };
                        notifier.create_info(String::from(&format!(
                            "Player {} selected position: {}",
                            player_position.user.get_username(),
                            position_str
                        )));
                    }
                    .boxed()
                })
                .on(JoinRoomNotification::MSG_TYPE, move |payload, _| {
                    let notifier = notifier_clone_5.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<JoinRoomNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        notifier.create_info(String::from(&format!(
                            "Player {} joined the room.",
                            msg.user.get_username()
                        )));
                    }
                    .boxed()
                })
                .on(LeaveRoomResponse::MSG_TYPE, move |payload, c| {
                    let client = client_clone_6.clone();
                    async move {
                        let _msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<LeaveRoomResponse>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_lock = client.lock().await;
                            client_lock.state = GuiClientState::InLobby;
                            client_lock.seats = [None, None, None, None];
                            client_lock.selected_seat = None;
                        }
                        c.emit(
                            ListRoomsMessage::MSG_TYPE,
                            to_string(&ListRoomsMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    .boxed()
                })
                .on(LeaveRoomNotification::MSG_TYPE, move |payload, _| {
                    let client = client_clone_7.clone();
                    let notifier = notifier_clone_6.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<LeaveRoomNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        let mut client_lock = client.lock().await;
                        update_user_seat(&mut client_lock.seats, msg.user.clone(), None);
                        notifier.create_info(String::from(&format!(
                            "Player {} left the room.",
                            msg.user.get_username()
                        )));
                    }
                    .boxed()
                })
                .on(GameStartedNotification::MSG_TYPE, move |payload, c| {
                    let client = client_clone_8.clone();
                    let notifier = notifier_clone_7.clone();
                    async move {
                        match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<GameStartedNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_lock = client.lock().await;
                            client_lock.state = GuiClientState::Playing;
                        }
                        notifier.create_info(String::from("Game started"));
                        c.emit(
                            GetCardsMessage::MSG_TYPE,
                            to_string(&GetCardsMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    .boxed()
                })
                .on(GetCardsResponse::MSG_TYPE, move |payload, _| {
                    let client = client_clone_9.clone();
                    let notifier = notifier_clone_8.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<GetCardsResponse>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
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
                                notifier
                                    .create_error(String::from("Spectator is not allowed to play"));
                            }
                        };
                    }
                    .boxed()
                })
                .on(AskBidNotification::MSG_TYPE, move |payload, _| {
                    let client = client_clone_10.clone();
                    let notifier = notifier_clone_9.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<AskBidNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
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
                    .boxed()
                })
                .on(MakeBidResponse::MSG_TYPE, move |payload, _| {
                    let client = client_clone_11.clone();
                    let notifier = notifier_clone_10.clone();
                    let input_placed_bid_arc = input_placed_bid_clone.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<MakeBidResponse>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        match msg {
                            MakeBidResponse::Ok => {
                                let input_placed_bid = input_placed_bid_arc.lock().await;

                                let mut client_lock = client.lock().await;
                                client_lock.placed_bid = *input_placed_bid;
                            }
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
                    .boxed()
                })
                .on(AuctionFinishedNotification::MSG_TYPE, move |payload, _| {
                    let client = client_clone_12.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => serde_json::from_value::<
                                AuctionFinishedNotification,
                            >(text[0].clone())
                            .unwrap(),
                            _ => return,
                        };
                        let msg = match msg {
                            AuctionFinishedNotification::Winner(msg) => msg,
                            AuctionFinishedNotification::NoWinner => {
                                panic!("No winner in auction")
                            }
                        };
                        {
                            let mut client_lock = client.lock().await;
                            client_lock.game_max_bid = Some(msg.max_bid);
                            client_lock.game_current_player = Some(msg.winner);
                        }
                    }
                    .boxed()
                })
                .on(DummyCardsNotification::MSG_TYPE, move |payload, _| {
                    let client = client_clone_13.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<DummyCardsNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_lock = client.lock().await;
                            client_lock.dummy_cards = Some(msg.cards);
                            client_lock.dummy_player = Some(msg.dummy);
                        }
                    }
                    .boxed()
                })
                .on(AskTrickNotification::MSG_TYPE, move |payload, _| {
                    let client = client_clone_14.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<AskTrickNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        // Remove the card if it was dummy's
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
                    .boxed()
                })
                .on(MakeTrickResponse::MSG_TYPE, move |payload, _| {
                    let client = client_clone_15.clone();
                    let notifier = notifier_clone_11.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<MakeTrickResponse>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
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
                    .boxed()
                })
                .on(TrickFinishedNotification::MSG_TYPE, move |payload, _| {
                    let client = client_clone_16.clone();
                    let notifier = notifier_clone_12.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<TrickFinishedNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_lock = client.lock().await;

                            // Remove the final card if it was dummy's
                            if let Some(dummy_cards) = client_lock.dummy_cards.as_mut() {
                                if let Some(card) = msg.cards.last() {
                                    dummy_cards.retain(|c| c != card);
                                }
                            }

                            // Add the final card to the placed_cards.
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
                    .boxed()
                })
                .on(GameFinishedNotification::MSG_TYPE, move |payload, c| {
                    let client = client_clone_17.clone();
                    let notifier = notifier_clone_13.clone();
                    async move {
                        let _msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<GameFinishedNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        notifier.create_info(String::from("Game finished!"));
                        sleep(Duration::from_secs(5)).await;
                        {
                            let mut client_lock = client.lock().await;
                            client_lock.state = GuiClientState::InLobby;
                            client_lock.selected_room_name = None;
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
                        c.emit(
                            ListRoomsMessage::MSG_TYPE,
                            to_string(&ListRoomsMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    .boxed()
                })
                .connect()
                .await
                .expect("Connection failed"),
        )
    });

    loop {
        clear_background(Color::from_rgba(50, 115, 85, 255));

        let mut client_lock = client.blocking_lock();
        let current_state = client_lock.state;

        match current_state {
            GuiClientState::Logging => {
                login_ui(socket.clone(), &runtime, input_nickname.clone());
            }
            GuiClientState::InLobby => {
                list_rooms(socket.clone(), &runtime, &mut client_lock);
            }
            GuiClientState::CreatingRoom => {
                create_room_ui(socket.clone(), &runtime, input_created_room_name.clone());
            }
            GuiClientState::InRoom => {
                room_ui(
                    socket.clone(),
                    &runtime,
                    client_lock.selected_room_name.clone(),
                    client_lock.seats.clone(),
                );
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
