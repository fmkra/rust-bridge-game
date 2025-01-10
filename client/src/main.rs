mod gui_client;
mod gui_create_room;
mod gui_lobby;
mod gui_login;
mod gui_notification;
mod gui_play;
mod gui_room;
mod utils;

use gui_client::{GuiClient, GuiClientState};
use gui_create_room::create_room_ui;
use gui_lobby::list_rooms;
use gui_login::login_ui;
use gui_notification::{
    create_error_notification, create_info_notification, display_notifications, Notification,
    NotificationType,
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
use utils::update_user_seat;

#[macroquad::main("Bridge card game")]
async fn main() {
    let bid_textures = preload_textures().await;
    let card_textures = preload_cards().await;
    let client = GuiClient::new();
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

    // Clones of GuiClient Arc fields
    let client_notifications_clone = client.notifications.clone();
    let client_notifications_clone_1 = client.notifications.clone();
    let client_notifications_clone_2 = client.notifications.clone();
    let client_notifications_clone_3 = client.notifications.clone();
    let client_notifications_clone_4 = client.notifications.clone();
    let client_notifications_clone_5 = client.notifications.clone();
    let client_notifications_clone_6 = client.notifications.clone();
    let client_notifications_clone_7 = client.notifications.clone();
    let client_notifications_clone_8 = client.notifications.clone();
    let client_notifications_clone_9 = client.notifications.clone();
    let client_notifications_clone_10 = client.notifications.clone();
    let client_notifications_clone_11 = client.notifications.clone();
    let client_notifications_clone_12 = client.notifications.clone();
    let client_notifications_clone_13 = client.notifications.clone();
    let client_name_clone = client.name.clone();
    let client_state_clone = client.state.clone();
    let client_state_clone_1 = client.state.clone();
    let client_state_clone_2 = client.state.clone();
    let client_state_clone_3 = client.state.clone();
    let client_rooms_clone = client.rooms.clone();
    let client_selected_room_name_clone = client.selected_room_name.clone();
    let client_seats_clone = client.seats.clone();
    let client_seats_clone_1 = client.seats.clone();
    let client_seats_clone_2 = client.seats.clone();
    let client_seats_clone_3 = client.seats.clone();
    let client_seats_clone_4 = client.seats.clone();
    let client_selected_seat_clone = client.selected_seat.clone();
    let client_selected_seat_clone_1 = client.selected_seat.clone();
    let client_selected_seat_clone_2 = client.selected_seat.clone();
    let client_card_list_clone = client.card_list.clone();
    let client_card_list_clone_1 = client.card_list.clone();
    let client_placed_bid_clone = client.placed_bid.clone();
    let client_placed_trick_clone = client.placed_trick.clone();
    let client_game_max_bid_clone = client.game_max_bid.clone();
    let client_game_current_player_clone = client.game_current_player.clone();
    let client_game_current_player_clone_1 = client.game_current_player.clone();
    let client_game_current_player_clone_2 = client.game_current_player.clone();
    let client_game_current_player_clone_3 = client.game_current_player.clone();
    let client_dummy_cards_clone = client.dummy_cards.clone();
    let client_dummy_cards_clone_1 = client.dummy_cards.clone();
    let client_dummy_player_clone = client.dummy_player.clone();
    let client_current_placed_cards_clone = client.current_placed_cards.clone();
    let client_current_placed_cards_clone_1 = client.current_placed_cards.clone();

    let client_state_clone_4 = client.state.clone();
    let client_selected_room_name_clone_1 = client.selected_room_name.clone();
    let client_selected_seat_clone_3 = client.selected_seat.clone();
    let client_card_list_clone_2 = client.card_list.clone();
    let client_player_bids_clone_1 = client.player_bids.clone();
    let client_placed_bid_clone_1 = client.placed_bid.clone();
    let client_placed_trick_clone_1 = client.placed_trick.clone();
    let client_game_max_bid_clone_1 = client.game_max_bid.clone();
    let client_game_current_player_clone_4 = client.game_current_player.clone();
    let client_dummy_cards_clone_2 = client.dummy_cards.clone();
    let client_dummy_cards_clone_3 = client.dummy_cards.clone();
    let client_dummy_player_clone_1 = client.dummy_player.clone();
    let client_current_placed_cards_clone_2 = client.current_placed_cards.clone();

    // Connect to the server
    let socket = runtime.block_on(async {
        Arc::new(
            ClientBuilder::new("http://localhost:3000/")
                .namespace("/")
                .on(LoginResponse::MSG_TYPE, move |payload, c| {
                    let notifications = client_notifications_clone.clone();
                    let name = client_name_clone.clone();
                    let state = client_state_clone.clone();
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
                                // Change the name
                                {
                                    let mut name_val = name.lock().await;
                                    let nickname_val = nickname.lock().await;
                                    *name_val = Some(nickname_val.clone());
                                }

                                // Change the state
                                {
                                    let mut state_val = state.lock().await;
                                    *state_val = GuiClientState::InLobby;
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
                                    notifications,
                                );
                            }
                            LoginResponse::UserAlreadyLoggedIn => {
                                create_error_notification(
                                    String::from("User is already logged in"),
                                    notifications,
                                );
                            }
                        }
                    }
                    .boxed()
                })
                .on(ListRoomsResponse::MSG_TYPE, move |payload, _| {
                    let rooms_arc = client_rooms_clone.clone();
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
                        let mut rooms_lock = rooms_arc.lock().await;
                        *rooms_lock = rooms;
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
                    let notifications = client_notifications_clone_2.clone();
                    let state_arc = client_state_clone_2.clone();
                    let client_selected_room_name_arc = client_selected_room_name_clone.clone();
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
                                    let mut state_val = state_arc.lock().await;
                                    *state_val = GuiClientState::InRoom;
                                }
                                {
                                    let mut client_selected_room_name_val =
                                        client_selected_room_name_arc.lock().await;
                                    let input_selected_room_name_val =
                                        input_selected_room_name_arc.lock().await;
                                    *client_selected_room_name_val =
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
                                    notifications,
                                );
                            }
                            JoinRoomResponse::AlreadyInRoom => {
                                create_error_notification(
                                    String::from("You are already in the room"),
                                    notifications,
                                );
                            }
                            JoinRoomResponse::RoomNotFound => {
                                create_error_notification(
                                    String::from("Room not found"),
                                    notifications,
                                );
                            }
                        }
                    }
                    .boxed()
                })
                .on(ListPlacesResponse::MSG_TYPE, move |payload, _| {
                    let notifications = client_notifications_clone_1.clone();
                    let seats_arc = client_seats_clone.clone();
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
                                let mut seats_val = seats_arc.lock().await;
                                *seats_val = msg;
                            }
                            ListPlacesResponse::NotInRoom => {
                                create_error_notification(
                                    String::from("You are not in a room"),
                                    notifications,
                                );
                            }
                            ListPlacesResponse::Unauthenticated => {
                                create_error_notification(
                                    String::from("You are not authenticated"),
                                    notifications,
                                );
                            }
                        }
                    }
                    .boxed()
                })
                .on(SelectPlaceResponse::MSG_TYPE, move |payload, c| {
                    let notifications = client_notifications_clone_3.clone();
                    let client_selected_seat_arc = client_selected_seat_clone.clone();
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
                                            let mut client_selected_seat_val =
                                                client_selected_seat_arc.lock().await;
                                            let input_selected_seat_val =
                                                input_selected_seat_arc.lock().await;
                                            *client_selected_seat_val = *input_selected_seat_val;
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
                                            notifications,
                                        );
                                    }
                                    SelectPlaceResponse::PlaceAlreadyTaken => {
                                        create_error_notification(
                                            String::from("Place is already taken"),
                                            notifications,
                                        );
                                    }
                                    SelectPlaceResponse::Unauthenticated => {
                                        create_error_notification(
                                            String::from("You are not authenticated"),
                                            notifications,
                                        );
                                    }
                                };
                            }
                            _ => return,
                        };
                    }
                    .boxed()
                })
                .on(SelectPlaceNotification::MSG_TYPE, move |payload, _| {
                    let notifications = client_notifications_clone_4.clone();
                    let client_seats = client_seats_clone_1.clone();
                    async move {
                        let player_position = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<SelectPlaceNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_seats_val = client_seats.lock().await;
                            update_user_seat(
                                &mut client_seats_val,
                                player_position.user.clone(),
                                player_position.position,
                            );
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
                            notifications,
                        );
                    }
                    .boxed()
                })
                .on(JoinRoomNotification::MSG_TYPE, move |payload, _| {
                    let notifications = client_notifications_clone_5.clone();
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
                            notifications,
                        );
                    }
                    .boxed()
                })
                .on(LeaveRoomResponse::MSG_TYPE, move |payload, c| {
                    let client_state = client_state_clone_1.clone();
                    let client_seats = client_seats_clone_2.clone();
                    let client_selected_seat = client_selected_seat_clone_1.clone();
                    async move {
                        let _msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<LeaveRoomResponse>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_state_val = client_state.lock().await;
                            *client_state_val = GuiClientState::InLobby;
                        }
                        {
                            let mut client_seats_val = client_seats.lock().await;
                            *client_seats_val = [None, None, None, None];
                        }
                        {
                            let mut client_selected_seat_val = client_selected_seat.lock().await;
                            *client_selected_seat_val = None;
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
                    let notifications = client_notifications_clone_6.clone();
                    let client_seats = client_seats_clone_3.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<LeaveRoomNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        let mut client_seats_val = client_seats.lock().await;
                        update_user_seat(&mut client_seats_val, msg.user.clone(), None);
                        create_info_notification(
                            String::from(&format!(
                                "Player {} left the room.",
                                msg.user.get_username()
                            )),
                            notifications,
                        );
                    }
                    .boxed()
                })
                .on(GameStartedNotification::MSG_TYPE, move |payload, c| {
                    let notifications = client_notifications_clone_7.clone();
                    let state = client_state_clone_3.clone();
                    async move {
                        match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<GameStartedNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut state_val = state.lock().await;
                            *state_val = GuiClientState::Playing;
                        }
                        create_info_notification(String::from("Game started"), notifications);
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
                    let card_list = client_card_list_clone.clone();
                    let selected_seat = client_selected_seat_clone_2.clone();
                    let notifications = client_notifications_clone_8.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<GetCardsResponse>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        match msg {
                            GetCardsResponse::Ok { cards, position } => {
                                {
                                    let mut card_list_val = card_list.lock().await;
                                    *card_list_val = Some(cards);
                                }
                                {
                                    let mut selected_seat_val = selected_seat.lock().await;
                                    *selected_seat_val = Some(position);
                                }
                            }
                            GetCardsResponse::NotInRoom => {
                                create_error_notification(
                                    String::from("You are not in a room"),
                                    notifications,
                                );
                            }
                            GetCardsResponse::Unauthenticated => {
                                create_error_notification(
                                    String::from("You are not authenticated"),
                                    notifications,
                                );
                            }
                            GetCardsResponse::SpectatorNotAllowed => {
                                create_error_notification(
                                    String::from("Spectator is not allowed to play"),
                                    notifications,
                                );
                            }
                        };
                    }
                    .boxed()
                })
                .on(AskBidNotification::MSG_TYPE, move |payload, _| {
                    let notifications = client_notifications_clone_9.clone();
                    let client_game_current_player_arc = client_game_current_player_clone.clone();
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
                        {
                            let mut client_game_current_player_val =
                                client_game_current_player_arc.lock().await;
                            *client_game_current_player_val = Some(msg.player);
                        }
                        create_info_notification(bid_message, notifications.clone());
                        create_info_notification(
                            String::from(&format!("Player {} is bidding right now.", msg.player)),
                            notifications,
                        );
                    }
                    .boxed()
                })
                .on(MakeBidResponse::MSG_TYPE, move |payload, _| {
                    let client_placed_bid_arc = client_placed_bid_clone.clone();
                    let input_placed_bid_arc = input_placed_bid_clone_1.clone();
                    let notifications = client_notifications_clone_10.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<MakeBidResponse>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        match msg {
                            MakeBidResponse::Ok => {
                                let mut client_placed_bid_val = client_placed_bid_arc.lock().await;
                                let input_placed_bid = input_placed_bid_arc.lock().await;
                                *client_placed_bid_val = *input_placed_bid;
                                println!("{:?}", *client_placed_bid_val);
                            }
                            MakeBidResponse::AuctionNotInProcess => {
                                create_error_notification(
                                    String::from("Auction is not in process"),
                                    notifications,
                                );
                            }
                            MakeBidResponse::NotInRoom => {
                                create_error_notification(
                                    String::from("You are not in a room"),
                                    notifications,
                                );
                            }
                            MakeBidResponse::Unauthenticated => {
                                create_error_notification(
                                    String::from("You are not authenticated"),
                                    notifications,
                                );
                            }
                            MakeBidResponse::SpectatorNotAllowed => {
                                create_error_notification(
                                    String::from("You are not allowed to play"),
                                    notifications,
                                );
                            }
                            MakeBidResponse::NotYourTurn => {
                                create_error_notification(
                                    String::from("It's not your turn"),
                                    notifications,
                                );
                            }
                            MakeBidResponse::InvalidBid => create_error_notification(
                                String::from("This bid is not valid"),
                                notifications,
                            ),
                        }
                    }
                    .boxed()
                })
                .on(AuctionFinishedNotification::MSG_TYPE, move |payload, _| {
                    let client_game_max_bid_arc = client_game_max_bid_clone.clone();
                    let client_game_current_player_arc = client_game_current_player_clone_1.clone();
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
                            let mut client_game_max_bid_val = client_game_max_bid_arc.lock().await;
                            *client_game_max_bid_val = Some(msg.max_bid);
                        }
                        {
                            let mut client_game_current_player_val =
                                client_game_current_player_arc.lock().await;
                            *client_game_current_player_val = Some(msg.winner);
                        }
                    }
                    .boxed()
                })
                .on(DummyCardsNotification::MSG_TYPE, move |payload, _| {
                    let client_dummy_cards_arc = client_dummy_cards_clone.clone();
                    let client_dummy_player_arc = client_dummy_player_clone.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<DummyCardsNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_dummy_cards_val = client_dummy_cards_arc.lock().await;
                            *client_dummy_cards_val = Some(msg.cards);
                        }
                        {
                            let mut client_dummy_player_val = client_dummy_player_arc.lock().await;
                            *client_dummy_player_val = Some(msg.dummy);
                        }
                    }
                    .boxed()
                })
                .on(AskTrickNotification::MSG_TYPE, move |payload, _| {
                    let client_dummy_cards_arc = client_dummy_cards_clone_1.clone();
                    let client_game_current_player_arc = client_game_current_player_clone_2.clone();
                    let client_current_placed_cards_arc = client_current_placed_cards_clone.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<AskTrickNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        // Remove the card if it was dummy's
                        {
                            let mut client_dummy_cards_val = client_dummy_cards_arc.lock().await;
                            let client_dummy_cards_val_clone = client_dummy_cards_val.clone();

                            if let Some(mut dummy_cards) = client_dummy_cards_val_clone {
                                if let Some(card) = msg.cards.last() {
                                    dummy_cards.retain(|c| c != card);
                                }
                                *client_dummy_cards_val = Some(dummy_cards);
                            }
                        }
                        {
                            let mut client_game_current_player_val =
                                client_game_current_player_arc.lock().await;
                            *client_game_current_player_val = Some(msg.player);
                        }
                        {
                            let mut client_current_placed_cards_val =
                                client_current_placed_cards_arc.lock().await;
                            let mut placed_cards: [Option<Card>; 4] = [None, None, None, None];
                            let mut previous_player = msg.player.prev();
                            for el in msg.cards.iter().rev() {
                                placed_cards[previous_player.to_usize()] = Some(el.clone());
                                previous_player = previous_player.prev();
                            }
                            *client_current_placed_cards_val = placed_cards;
                        }
                    }
                    .boxed()
                })
                .on(MakeTrickResponse::MSG_TYPE, move |payload, _| {
                    let notifications = client_notifications_clone_11.clone();
                    let client_card_list_arc = client_card_list_clone_1.clone();
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
                                let mut client_card_list_val = client_card_list_arc.lock().await;
                                let input_placed_trick_val = input_placed_trick_arc.lock().await;
                                let client_card_list_val_clone = client_card_list_val.clone();
                                if let Some(mut cards) = client_card_list_val_clone {
                                    if let Some(placed_card) = *input_placed_trick_val {
                                        cards.retain(|c| *c != placed_card);
                                        *client_card_list_val = Some(cards);
                                    }
                                }
                            }
                            MakeTrickResponse::NotInRoom => {
                                create_error_notification(
                                    String::from("You are not in a room"),
                                    notifications,
                                );
                            }
                            MakeTrickResponse::SpectatorNotAllowed => {
                                create_error_notification(
                                    String::from("You are not allowed to play"),
                                    notifications,
                                );
                            }
                            MakeTrickResponse::NotYourTurn => {
                                create_error_notification(
                                    String::from("It's not your turn"),
                                    notifications,
                                );
                            }
                            MakeTrickResponse::TrickNotInProcess => {
                                create_error_notification(
                                    String::from("Trick is not in process"),
                                    notifications,
                                );
                            }
                            MakeTrickResponse::InvalidCard => {
                                create_error_notification(
                                    String::from("This card is not valid"),
                                    notifications,
                                );
                            }
                            MakeTrickResponse::Unauthenticated => {
                                create_error_notification(
                                    String::from("You are not authenticated"),
                                    notifications,
                                );
                            }
                        }
                    }
                    .boxed()
                })
                .on(TrickFinishedNotification::MSG_TYPE, move |payload, _| {
                    let client_game_current_player_arc = client_game_current_player_clone_3.clone();
                    let client_current_placed_cards_arc =
                        client_current_placed_cards_clone_1.clone();
                    let notifications = client_notifications_clone_12.clone();
                    let client_dummy_cards_arc = client_dummy_cards_clone_2.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<TrickFinishedNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        // Remove the final card if it was dummy's
                        {
                            let mut client_dummy_cards_val = client_dummy_cards_arc.lock().await;
                            let client_dummy_cards_val_clone = client_dummy_cards_val.clone();

                            if let Some(mut dummy_cards) = client_dummy_cards_val_clone {
                                if let Some(card) = msg.cards.last() {
                                    dummy_cards.retain(|c| c != card);
                                }
                                *client_dummy_cards_val = Some(dummy_cards);
                            }
                        }
                        // Add the final card to the placed_cards.
                        {
                            let current_player = {
                                let val = client_game_current_player_arc.lock().await;
                                *val
                            };
                            if let Some(current_player) = current_player {
                                let mut client_current_placed_cards_val =
                                    client_current_placed_cards_arc.lock().await;
                                let mut placed_cards = *client_current_placed_cards_val;
                                if let Some(last_tricked) = msg.cards.last() {
                                    placed_cards[current_player.to_usize()] = Some(*last_tricked);
                                }
                                *client_current_placed_cards_val = placed_cards;
                            }
                        }
                        create_info_notification(
                            String::from(format!(
                                "Trick {} taken by {:?}",
                                msg.cards
                                    .iter()
                                    .map(Card::to_string)
                                    .collect::<Vec<_>>()
                                    .join(" "),
                                msg.taker
                            )),
                            notifications,
                        );
                    }
                    .boxed()
                })
                .on(GameFinishedNotification::MSG_TYPE, move |payload, c| {
                    let notifications = client_notifications_clone_13.clone();
                    let client_state = client_state_clone_4.clone();
                    let selected_room_name = client_selected_room_name_clone_1.clone();
                    let client_seats = client_seats_clone_4.clone();
                    let client_selected_seat = client_selected_seat_clone_3.clone();
                    let client_card_list = client_card_list_clone_2.clone();
                    let client_player_bids = client_player_bids_clone_1.clone();
                    let client_placed_bid = client_placed_bid_clone_1.clone();
                    let client_placed_trick = client_placed_trick_clone_1.clone();
                    let client_game_max_bid = client_game_max_bid_clone_1.clone();
                    let client_game_current_player = client_game_current_player_clone_4.clone();
                    let client_dummy_cards = client_dummy_cards_clone_3.clone();
                    let client_dummy_player = client_dummy_player_clone_1.clone();
                    let client_current_placed_cards = client_current_placed_cards_clone_2.clone();
                    async move {
                        let _msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<GameFinishedNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        create_info_notification(String::from("Game finished!"), notifications);
                        sleep(Duration::from_secs(5)).await;
                        {
                            {
                                *client_state.lock().await = GuiClientState::InLobby;
                            }
                            {
                                *selected_room_name.lock().await = None;
                            }
                            {
                                *client_seats.lock().await = [None, None, None, None];
                            }
                            {
                                *client_selected_seat.lock().await = None;
                            }
                            {
                                *client_card_list.lock().await = None;
                            }
                            {
                                *client_player_bids.lock().await = [None, None, None, None];
                            }
                            {
                                *client_placed_bid.lock().await = None;
                            }
                            {
                                *client_placed_trick.lock().await = None;
                            }
                            {
                                *client_game_max_bid.lock().await = None;
                            }
                            {
                                *client_game_current_player.lock().await = None;
                            }
                            {
                                *client_dummy_cards.lock().await = None;
                            }
                            {
                                *client_dummy_player.lock().await = None;
                            }
                            {
                                *client_current_placed_cards.lock().await =
                                    [None, None, None, None];
                            }
                            {}
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

        let current_state = {
            let state_lock = client.state.lock().await;
            *state_lock
        };

        match current_state {
            GuiClientState::Logging => {
                login_ui(socket.clone(), &runtime, input_nickname.clone());
            }
            GuiClientState::InLobby => {
                list_rooms(
                    socket.clone(),
                    &runtime,
                    client.rooms.clone(),
                    client.state.clone(),
                    input_selected_room_name.clone(),
                );
            }
            GuiClientState::CreatingRoom => {
                create_room_ui(socket.clone(), &runtime, input_created_room_name.clone());
            }
            GuiClientState::InRoom => {
                room_ui(
                    socket.clone(),
                    &runtime,
                    client.selected_room_name.clone(),
                    client.seats.clone(),
                );
            }
            GuiClientState::Playing => {
                play_ui(
                    socket.clone(),
                    &runtime,
                    client.selected_seat.clone(),
                    client.seats.clone(),
                    client.card_list.clone(),
                    input_placed_bid_clone.clone(),
                    input_placed_trick.clone(),
                    client.game_current_player.clone(),
                    client.dummy_cards.clone(),
                    client.dummy_player.clone(),
                    client.current_placed_cards.clone(),
                    &bid_textures,
                    &card_textures,
                );
            }
        }

        display_notifications(client.notifications.clone()).await;

        next_frame().await;
    }
}
