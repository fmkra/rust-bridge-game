mod gui_client;
mod gui_create_room;
mod gui_lobby;
mod gui_login;
mod gui_notification;
mod gui_play;
mod gui_room;

use gui_client::{GuiClient, GuiClientState};
use gui_create_room::create_room_ui;
use gui_lobby::list_rooms;
use gui_login::login_ui;
use gui_notification::{
    create_error_notification, create_info_notification, display_notifications,
};
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
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde_json::to_string;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::{runtime::Runtime, time::sleep};

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
    let input_placed_trick: Arc<Mutex<Option<Card>>> = Arc::new(Mutex::new(None));
    let input_placed_trick_clone = input_placed_trick.clone();
    // let input_selected_bid =

    // Connect to the server
    let socket = runtime.block_on(async {
        Arc::new(
            ClientBuilder::new("http://localhost:3000/")
                .namespace("/")
                .on(LoginResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, c| {
                        let client = client.clone();
                        let nickname = input_nickname_clone.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<LoginResponse>(text[0].clone())
                                        .unwrap()
                                }
                                _ => return,
                            };

                            match msg {
                                LoginResponse::Ok => {
                                    println!("Login successful!");
                                    // Change the name
                                    {
                                        let nickname_val = nickname.lock().await.clone();
                                        let mut client_lock = client.lock().await;
                                        client_lock.name = Some(nickname_val);
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
                                    create_error_notification(
                                        String::from("Username already exists"),
                                        client,
                                    );
                                }
                                LoginResponse::UserAlreadyLoggedIn => {
                                    create_error_notification(
                                        String::from("User is already logged in"),
                                        client,
                                    );
                                }
                            }
                        }
                        .boxed()
                    }
                })
                .on(ListRoomsResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = Arc::clone(&client);
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
                            client.lock().await.rooms = rooms;
                        }
                        .boxed()
                    }
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
                .on(JoinRoomResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, c| {
                        let client = client.clone();
                        let input_selected_room_name_arc = input_selected_room_name_clone_1.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<JoinRoomResponse>(text[0].clone())
                                        .unwrap()
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
                                    create_error_notification(
                                        String::from("You are not authenticated"),
                                        client,
                                    );
                                }
                                JoinRoomResponse::AlreadyInRoom => {
                                    create_error_notification(
                                        String::from("You are already in the room"),
                                        client,
                                    );
                                }
                                JoinRoomResponse::RoomNotFound => {
                                    create_error_notification(
                                        String::from("Room not found"),
                                        client,
                                    );
                                }
                            }
                        }
                        .boxed()
                    }
                })
                .on(ListPlacesResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
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
                                    client.lock().await.seats = msg;
                                }
                                ListPlacesResponse::NotInRoom => {
                                    create_error_notification(
                                        String::from("You are not in a room"),
                                        client,
                                    );
                                }
                                ListPlacesResponse::Unauthenticated => {
                                    create_error_notification(
                                        String::from("You are not authenticated"),
                                        client,
                                    );
                                }
                            }
                        }
                        .boxed()
                    }
                })
                .on(SelectPlaceResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, c| {
                        let client = client.clone();
                        let input_selected_seat_arc = input_selected_seat_clone.clone();
                        async move {
                            if let Payload::Text(text) = payload {
                                let msg =
                                    serde_json::from_value::<SelectPlaceResponse>(text[0].clone())
                                        .unwrap();
                                match msg {
                                    SelectPlaceResponse::Ok => {
                                        {
                                            let input_selected_seat_val =
                                                input_selected_seat_arc.lock().await;

                                            client.lock().await.selected_seat =
                                                *input_selected_seat_val;
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
                                        create_error_notification(
                                            String::from("You are not in a room"),
                                            client,
                                        );
                                    }
                                    SelectPlaceResponse::PlaceAlreadyTaken => {
                                        create_error_notification(
                                            String::from("Place is already taken"),
                                            client,
                                        );
                                    }
                                    SelectPlaceResponse::Unauthenticated => {
                                        create_error_notification(
                                            String::from("You are not authenticated"),
                                            client,
                                        );
                                    }
                                };
                            };
                        }
                        .boxed()
                    }
                })
                .on(SelectPlaceNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        async move {
                            let player_position = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<SelectPlaceNotification>(
                                        text[0].clone(),
                                    )
                                    .unwrap()
                                }
                                _ => return,
                            };
                            {
                                let mut client_seats_val = client.lock().await.seats.clone();
                                if let Some(seat) = player_position.position {
                                    client_seats_val[seat.to_usize()] =
                                        Some(player_position.user.clone());
                                }
                            }
                            let position_str = match player_position.position {
                                Some(val) => format!("{}", val),
                                None => String::from("Spectator"),
                            };
                            create_info_notification(
                                String::from(&format!(
                                    "Player {} selected position: {}",
                                    player_position.user.get_username(),
                                    position_str
                                )),
                                client,
                            );
                        }
                        .boxed()
                    }
                })
                .on(JoinRoomNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<JoinRoomNotification>(text[0].clone())
                                        .unwrap()
                                }
                                _ => return,
                            };
                            create_info_notification(
                                String::from(&format!(
                                    "Player {} joined the room.",
                                    msg.user.get_username()
                                )),
                                client,
                            );
                        }
                        .boxed()
                    }
                })
                .on(LeaveRoomResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, c| {
                        let client = client.clone();
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
                    }
                })
                .on(LeaveRoomNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<LeaveRoomNotification>(text[0].clone())
                                        .unwrap()
                                }
                                _ => return,
                            };
                            create_info_notification(
                                String::from(&format!(
                                    "Player {} left the room.",
                                    msg.user.get_username()
                                )),
                                client,
                            );
                        }
                        .boxed()
                    }
                })
                .on(GameStartedNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, c| {
                        let client = client.clone();
                        async move {
                            match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<GameStartedNotification>(
                                        text[0].clone(),
                                    )
                                    .unwrap()
                                }
                                _ => return,
                            };
                            client.lock().await.state = GuiClientState::Playing;
                            create_info_notification(String::from("Game started"), client);
                            c.emit(
                                GetCardsMessage::MSG_TYPE,
                                to_string(&GetCardsMessage {}).unwrap(),
                            )
                            .await
                            .unwrap();
                        }
                        .boxed()
                    }
                })
                .on(GetCardsResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<GetCardsResponse>(text[0].clone())
                                        .unwrap()
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
                                    create_error_notification(
                                        String::from("You are not in a room"),
                                        client,
                                    );
                                }
                                GetCardsResponse::Unauthenticated => {
                                    create_error_notification(
                                        String::from("You are not authenticated"),
                                        client,
                                    );
                                }
                                GetCardsResponse::SpectatorNotAllowed => {
                                    create_error_notification(
                                        String::from("Spectator is not allowed to play"),
                                        client,
                                    );
                                }
                            };
                        }
                        .boxed()
                    }
                })
                .on(AskBidNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
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
                                _ => format!("Current max bid is {}", msg.max_bid),
                            };
                            client.lock().await.game_current_player = Some(msg.player);
                            create_info_notification(bid_message, client.clone());
                            create_info_notification(
                                String::from(&format!(
                                    "Player {} is bidding right now.",
                                    msg.player
                                )),
                                client,
                            );
                        }
                        .boxed()
                    }
                })
                .on(MakeBidResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        let input_placed_bid_arc = input_placed_bid_clone_1.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<MakeBidResponse>(text[0].clone())
                                        .unwrap()
                                }
                                _ => return,
                            };
                            match msg {
                                MakeBidResponse::Ok => {
                                    let mut client_lock = client.lock().await;
                                    let input_placed_bid = input_placed_bid_arc.lock().await;
                                    client_lock.placed_bid = *input_placed_bid;
                                }
                                MakeBidResponse::AuctionNotInProcess => {
                                    create_error_notification(
                                        String::from("Auction is not in process"),
                                        client,
                                    );
                                }
                                MakeBidResponse::NotInRoom => {
                                    create_error_notification(
                                        String::from("You are not in a room"),
                                        client,
                                    );
                                }
                                MakeBidResponse::Unauthenticated => {
                                    create_error_notification(
                                        String::from("You are not authenticated"),
                                        client,
                                    );
                                }
                                MakeBidResponse::SpectatorNotAllowed => {
                                    create_error_notification(
                                        String::from("You are not allowed to play"),
                                        client,
                                    );
                                }
                                MakeBidResponse::NotYourTurn => {
                                    create_error_notification(
                                        String::from("It's not your turn"),
                                        client,
                                    );
                                }
                                MakeBidResponse::InvalidBid => create_error_notification(
                                    String::from("This bid is not valid"),
                                    client,
                                ),
                            }
                        }
                        .boxed()
                    }
                })
                .on(AuctionFinishedNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<AuctionFinishedNotification>(
                                        text[0].clone(),
                                    )
                                    .unwrap()
                                }
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
                    }
                })
                .on(DummyCardsNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<DummyCardsNotification>(
                                        text[0].clone(),
                                    )
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
                    }
                })
                .on(AskTrickNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<AskTrickNotification>(text[0].clone())
                                        .unwrap()
                                }
                                _ => return,
                            };
                            {
                                let mut client_lock = client.lock().await;

                                // Remove the card if it was dummy's
                                if let Some(mut dummy_cards) = client_lock.dummy_cards.clone() {
                                    if let Some(card) = msg.cards.last() {
                                        dummy_cards.retain(|c| c != card);
                                    }
                                    client_lock.dummy_cards = Some(dummy_cards);
                                }

                                client_lock.game_current_player = Some(msg.player);
                                let mut placed_cards: [Option<Card>; 4] = [None, None, None, None];
                                let mut previous_player = msg.player.prev();
                                for el in msg.cards.iter().rev() {
                                    placed_cards[previous_player.to_usize()] = Some(*el);
                                    previous_player = previous_player.prev();
                                }
                                client_lock.current_placed_cards = placed_cards;
                            }
                        }
                        .boxed()
                    }
                })
                .on(MakeTrickResponse::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        let input_placed_trick_arc = input_placed_trick_clone.clone();
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
                                    let input_placed_trick_val =
                                        input_placed_trick_arc.lock().await;
                                    let mut client_lock = client.lock().await;
                                    if let Some(mut cards) = client_lock.card_list.clone() {
                                        if let Some(placed_card) = *input_placed_trick_val {
                                            cards.retain(|c| *c != placed_card);
                                            client_lock.card_list = Some(cards);
                                        }
                                    }
                                }
                                MakeTrickResponse::NotInRoom => {
                                    create_error_notification(
                                        String::from("You are not in a room"),
                                        client,
                                    );
                                }
                                MakeTrickResponse::SpectatorNotAllowed => {
                                    create_error_notification(
                                        String::from("You are not allowed to play"),
                                        client,
                                    );
                                }
                                MakeTrickResponse::NotYourTurn => {
                                    create_error_notification(
                                        String::from("It's not your turn"),
                                        client,
                                    );
                                }
                                MakeTrickResponse::TrickNotInProcess => {
                                    create_error_notification(
                                        String::from("Trick is not in process"),
                                        client,
                                    );
                                }
                                MakeTrickResponse::InvalidCard => {
                                    create_error_notification(
                                        String::from("This card is not valid"),
                                        client,
                                    );
                                }
                                MakeTrickResponse::Unauthenticated => {
                                    create_error_notification(
                                        String::from("You are not authenticated"),
                                        client,
                                    );
                                }
                            }
                        }
                        .boxed()
                    }
                })
                .on(TrickFinishedNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, _| {
                        let client = client.clone();
                        async move {
                            let msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<TrickFinishedNotification>(
                                        text[0].clone(),
                                    )
                                    .unwrap()
                                }
                                _ => return,
                            };
                            {
                                let mut client_lock = client.lock().await;

                                // Remove the final card if it was dummy's
                                if let Some(mut dummy_cards) = client_lock.dummy_cards.clone() {
                                    if let Some(card) = msg.cards.last() {
                                        dummy_cards.retain(|c| c != card);
                                    }
                                    client_lock.dummy_cards = Some(dummy_cards);
                                }

                                // Add the final card to the placed_cards.
                                let current_player = client_lock.game_current_player;
                                if let Some(current_player) = current_player {
                                    let mut placed_cards = client_lock.current_placed_cards;
                                    if let Some(last_tricked) = msg.cards.last() {
                                        placed_cards[current_player.to_usize()] =
                                            Some(*last_tricked);
                                    }
                                    client_lock.current_placed_cards = placed_cards;
                                }
                            }
                            create_info_notification(
                                format!(
                                    "Trick {} taken by {:?}",
                                    msg.cards
                                        .iter()
                                        .map(Card::to_string)
                                        .collect::<Vec<_>>()
                                        .join(" "),
                                    msg.taker
                                ),
                                client,
                            );
                        }
                        .boxed()
                    }
                })
                .on(GameFinishedNotification::MSG_TYPE, {
                    let client = Arc::clone(&client);
                    move |payload, c| {
                        let client = client.clone();
                        async move {
                            let _msg = match payload {
                                Payload::Text(text) => {
                                    serde_json::from_value::<GameFinishedNotification>(
                                        text[0].clone(),
                                    )
                                    .unwrap()
                                }
                                _ => return,
                            };
                            create_info_notification(
                                String::from("Game finished!"),
                                client.clone(),
                            );
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
                    }
                })
                .connect()
                .await
                .expect("Connection failed"),
        )
    });

    let client_clone = client.clone();
    loop {
        clear_background(Color::from_rgba(50, 115, 85, 255));

        let current_state = client_clone.lock().await.state;

        match current_state {
            GuiClientState::Logging => {
                login_ui(socket.clone(), &runtime, input_nickname.clone());
            }
            GuiClientState::InLobby => {
                list_rooms(socket.clone(), &runtime, client_clone.clone()).await;
            }
            GuiClientState::CreatingRoom => {
                create_room_ui(socket.clone(), &runtime, input_created_room_name.clone());
            }
            GuiClientState::InRoom => {
                room_ui(socket.clone(), &runtime, client_clone.clone()).await;
            }
            GuiClientState::Playing => {
                play_ui(
                    socket.clone(),
                    &runtime,
                    client_clone.clone(),
                    input_placed_bid_clone.clone(),
                    input_placed_trick.clone(),
                    &bid_textures,
                    &card_textures,
                )
                .await;
            }
        }

        display_notifications(client_clone.clone()).await;

        next_frame().await;
    }
}
