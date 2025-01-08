mod gui_login;
mod gui_notification;
mod gui_client;
mod gui_lobby;
mod gui_create_room;
mod gui_room;
mod gui_play;

use gui_client::{GuiClientState, GuiClient};
use gui_login::login_ui;
use gui_notification::{create_error_notification, create_info_notification, display_notifications, Notification, NotificationType};
use gui_lobby::list_rooms;
use gui_create_room::create_room_ui;
use gui_room::room_ui;
use gui_play::{play_ui, preload_textures, preload_cards};

use std::sync::Arc;
use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde_json::to_string;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use macroquad::prelude::*;
use common::{
    message::{
        client_message::{
            GetCardsMessage,
            JoinRoomMessage,
            ListPlacesMessage, 
            ListRoomsMessage,
            GET_CARDS_MESSAGE,
            JOIN_ROOM_MESSAGE,
            LIST_PLACES_MESSAGE,
            LIST_ROOMS_MESSAGE,
        }, server_notification::{
            AskBidNotification,
            GameStartedNotification,
            JoinRoomNotification,
            LeaveRoomNotification,
            SelectPlaceNotification,
            AuctionFinishedNotification,
            DummyCardsNotification,
            AskTrickNotification,
            ASK_BID_NOTIFICATION,
            GAME_STARTED_NOTIFICATION,
            JOIN_ROOM_NOTIFICATION,
            LEAVE_ROOM_NOTIFICATION,
            SELECT_PLACE_NOTIFICATION,
            AUCTION_FINISHED_NOTIFICATION,
            DUMMY_CARDS_NOTIFICATION,
            ASK_TRICK_NOTIFICATION,
        }, server_response::{
            GetCardsResponse,
            JoinRoomResponse,
            LeaveRoomResponse,
            ListPlacesResponse,
            ListRoomsResponse,
            LoginResponse,
            SelectPlaceResponse,
            MakeBidResponse,
            GET_CARDS_RESPONSE,
            JOIN_ROOM_RESPONSE,
            LEAVE_ROOM_RESPONSE,
            LIST_PLACES_RESPONSE,
            LIST_ROOMS_RESPONSE,
            LOGIN_RESPONSE,
            REGISTER_ROOM_RESPONSE,
            SELECT_PLACE_RESPONSE,
            MAKE_BID_RESPONSE,
        }
    }, room::RoomId, Bid, Player, Card
};

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
    let input_selected_room_name_clone: Arc<Mutex<Option<String>>> = input_selected_room_name.clone();
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
    let client_selected_seat_clone = client.selected_seat.clone();
    let client_selected_seat_clone_1 = client.selected_seat.clone();
    let client_selected_seat_clone_2 = client.selected_seat.clone();
    let client_card_list_clone = client.card_list.clone();
    let client_placed_bid_clone = client.placed_bid.clone();
    let client_placed_trick_clone = client.placed_trick.clone();
    let client_game_max_bid_clone = client.game_max_bid.clone();
    let client_game_current_player_clone = client.game_current_player.clone();
    let client_game_current_player_clone_1 = client.game_current_player.clone();
    let client_dummy_cards_clone = client.dummy_cards.clone();
    let client_dummy_cards_clone_1 = client.dummy_cards.clone();
    let client_dummy_player_clone = client.dummy_player.clone();

    // Connect to the server
    let socket = runtime.block_on(async {
        Arc::new(
            ClientBuilder::new("http://localhost:3000/")
                .namespace("/")
                .on(LOGIN_RESPONSE, move |payload, c| {
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

                                c.emit(LIST_ROOMS_MESSAGE, to_string(&ListRoomsMessage {}).unwrap())
                                    .await
                                    .unwrap();
                            }
                            LoginResponse::UsernameAlreadyExists => {
                                create_error_notification(String::from("Username already exists"), notifications);
                            }
                            LoginResponse::UserAlreadyLoggedIn => {
                                create_error_notification(String::from("User is already logged in"),notifications);
                            }
                        }
                    }
                    .boxed()
                })
                .on(LIST_ROOMS_RESPONSE, move |payload, _| {
                    let rooms_arc = client_rooms_clone.clone();
                    async move {
                        let rooms = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<ListRoomsResponse>(text[0].clone()).unwrap()
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
                .on(REGISTER_ROOM_RESPONSE, move |_, c| {
                    let input_selected_room_name_arc = input_selected_room_name_clone.clone();
                    let input_created_room_name_arc = input_created_room_name_clone.clone();
                    async move {
                        let room_id = {
                            let mut input_selected_room_name_val = input_selected_room_name_arc.lock().await;
                            let input_created_room_name_val = input_created_room_name_arc.lock().await;
                            *input_selected_room_name_val = Some(input_created_room_name_val.clone());
                            input_created_room_name_val.clone()
                        };
                        c.emit(JOIN_ROOM_MESSAGE, to_string(&JoinRoomMessage { room_id: RoomId::new(room_id.as_str()) }).unwrap())
                        .await
                        .unwrap();
                    }
                    .boxed()
                })
                .on(JOIN_ROOM_RESPONSE, move |payload, c| {
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
                                    let mut client_selected_room_name_val = client_selected_room_name_arc.lock().await;
                                    let input_selected_room_name_val = input_selected_room_name_arc.lock().await;
                                    *client_selected_room_name_val = input_selected_room_name_val.clone();
                                }
                                c.emit(
                                    LIST_PLACES_MESSAGE,
                                    to_string(&ListPlacesMessage {}).unwrap(),
                                )
                                .await
                                .unwrap();
                            },
                            JoinRoomResponse::Unauthenticated => {
                                create_error_notification(String::from("You are not authenticated"),notifications);
                            },
                            JoinRoomResponse::AlreadyInRoom => {
                                create_error_notification(String::from("You are already in the room"),notifications);
                            },
                            JoinRoomResponse::RoomNotFound => {
                                create_error_notification(String::from("Room not found"),notifications);
                            },
                        }
                    }
                    .boxed()
                })
                .on(LIST_PLACES_RESPONSE, move |payload, _| {
                    let notifications = client_notifications_clone_1.clone();
                    let seats_arc = client_seats_clone.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<ListPlacesResponse>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        match msg {
                            ListPlacesResponse::Ok(msg) => {
                                let mut seats_val = seats_arc.lock().await;
                                *seats_val = msg;
                            }
                            ListPlacesResponse::NotInRoom => {
                                create_error_notification(String::from("You are not in a room"),notifications);
                            }
                            ListPlacesResponse::Unauthenticated => {
                                create_error_notification(String::from("You are not authenticated"),notifications);
                            }
                        }
                    }
                    .boxed()
                })
                .on(SELECT_PLACE_RESPONSE, move |payload, c| {
                    let notifications = client_notifications_clone_3.clone();
                    let client_selected_seat_arc = client_selected_seat_clone.clone();
                    let input_selected_seat_arc = input_selected_seat_clone.clone();
                    async move {
                        match payload {
                            Payload::Text(text) => {
                                let msg = serde_json::from_value::<SelectPlaceResponse>(text[0].clone()).unwrap();
                                match msg {
                                    SelectPlaceResponse::Ok => {
                                        {
                                            let mut client_selected_seat_val = client_selected_seat_arc.lock().await;
                                            let input_selected_seat_val = input_selected_seat_arc.lock().await;
                                            *client_selected_seat_val = *input_selected_seat_val;
                                        };
                                        // Refreshes the seats
                                        c.emit(
                                            LIST_PLACES_MESSAGE,
                                            to_string(&ListPlacesMessage {}).unwrap(),
                                        )
                                        .await
                                        .unwrap();
                                    },
                                    SelectPlaceResponse::NotInRoom => {
                                        create_error_notification(String::from("You are not in a room"),notifications);
                                    },
                                    SelectPlaceResponse::PlaceAlreadyTaken => {
                                        create_error_notification(String::from("Place is already taken"),notifications);
                                    },
                                    SelectPlaceResponse::Unauthenticated => {
                                        create_error_notification(String::from("You are not authenticated"),notifications);
                                    },
                                };
                            }
                            _ => return,
                        };
                    }
                    .boxed()
                })
                .on(SELECT_PLACE_NOTIFICATION, move |payload, _| {
                    let notifications = client_notifications_clone_4.clone();
                    let client_seats = client_seats_clone_1.clone();
                    async move {
                        let player_position = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<SelectPlaceNotification>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut client_seats_val = client_seats.lock().await;
                            if let Some(seat) = player_position.position {
                                client_seats_val[seat.to_usize()] = Some(player_position.user.clone());
                            }
                        }
                        let position_str = match player_position.position {
                            Some(val) => format!("{}", val),
                            None => String::from("Spectator"),
                        };
                        create_info_notification(
                        String::from(
                                &format!(
                                    "Player {} selected position: {}", 
                                    player_position.user.get_username(),
                                    position_str
                                )
                            ),
                            notifications
                        );
                    }
                    .boxed()
                })
                .on(JOIN_ROOM_NOTIFICATION, move |payload, _| {
                    let notifications = client_notifications_clone_5.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<JoinRoomNotification>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        create_info_notification(
                            String::from(
                                &format!(
                                    "Player {} joined the room.", 
                                    msg.user.get_username()
                                )
                            ),
                            notifications
                        );
                    }
                    .boxed()
                })
                .on(LEAVE_ROOM_RESPONSE, move |payload, c| {
                    let client_state = client_state_clone_1.clone();
                    let client_seats = client_seats_clone_2.clone();
                    let client_selected_seat = client_selected_seat_clone_1.clone();
                    async move {
                        let _msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<LeaveRoomResponse>(text[0].clone()).unwrap()
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
                        c.emit(LIST_ROOMS_MESSAGE, to_string(&ListRoomsMessage {}).unwrap())
                            .await
                            .unwrap();
                    }
                    .boxed()
                })
                .on(LEAVE_ROOM_NOTIFICATION, move |payload, _| {
                    let notifications = client_notifications_clone_6.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<LeaveRoomNotification>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        create_info_notification(
                            String::from(
                                &format!(
                                    "Player {} left the room.", 
                                    msg.user.get_username()
                                )
                            ),
                            notifications
                        );
                    }
                    .boxed()
                })
                .on(GAME_STARTED_NOTIFICATION, move |payload, c| {
                    let notifications = client_notifications_clone_7.clone();
                    let state = client_state_clone_3.clone();
                    async move {
                        match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<GameStartedNotification>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        {
                            let mut state_val = state.lock().await;
                            *state_val = GuiClientState::Playing;
                        }
                        create_info_notification(String::from("Game started"),notifications);
                        c.emit(GET_CARDS_MESSAGE, to_string(&GetCardsMessage {}).unwrap())
                            .await
                            .unwrap();
                    }
                    .boxed()
                })
                .on(GET_CARDS_RESPONSE, move |payload, _| {
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
                            },
                            GetCardsResponse::NotInRoom => {
                                create_error_notification(String::from("You are not in a room"),notifications);
                            },
                            GetCardsResponse::Unauthenticated => {
                                create_error_notification(String::from("You are not authenticated"),notifications);
                            },
                            GetCardsResponse::SpectatorNotAllowed => {
                                create_error_notification(String::from("Spectator is not allowed to play"),notifications);
                            }
                        };
                    }
                    .boxed()
                })
                .on(ASK_BID_NOTIFICATION, move |payload, _| {
                    let notifications = client_notifications_clone_9.clone();
                    let client_game_current_player_arc = client_game_current_player_clone.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<AskBidNotification>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
                        let bid_message = match msg.max_bid {
                            Bid::Pass => {
                                String::from("There is no max bid")
                            },
                            _ => {
                                String::from(format!("Current max bid is {}", msg.max_bid))
                            },
                        };
                        {
                            let mut client_game_current_player_val = client_game_current_player_arc.lock().await;
                            *client_game_current_player_val = Some(msg.player);
                        }
                        create_info_notification(bid_message,notifications.clone());
                        create_info_notification(
                        String::from(
                                &format!(
                                    "Player {} is bidding right now.", 
                                    msg.player
                                )
                            ),
                            notifications
                        );
                    }
                    .boxed()
                })
                .on(MAKE_BID_RESPONSE, move |payload, _| {
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
                            },
                            MakeBidResponse::AuctionNotInProcess => {
                                create_error_notification(String::from("Auction is not in process"),notifications);
                            },
                            MakeBidResponse::NotInRoom => {
                                create_error_notification(String::from("You are not in a room"),notifications);
                            },
                            MakeBidResponse::Unauthenticated => {
                                create_error_notification(String::from("You are not authenticated"),notifications);
                            },
                            MakeBidResponse::SpectatorNotAllowed => {
                                create_error_notification(String::from("You are not allowed to play"),notifications);
                            },
                            MakeBidResponse::NotYourTurn => {
                                create_error_notification(String::from("It's not your turn"),notifications);
                            },
                            MakeBidResponse::InvalidBid => {
                                create_error_notification(String::from("This bid is not valid"),notifications)
                            }
                        }
                    }
                    .boxed()
                })
                .on(AUCTION_FINISHED_NOTIFICATION, move |payload, _| {
                    let client_game_max_bid_arc = client_game_max_bid_clone.clone();
                    let client_game_current_player_arc = client_game_current_player_clone_1.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<AuctionFinishedNotification>(text[0].clone())
                                    .unwrap()
                            }
                            _ => return,
                        };
                        let msg = msg.expect("No winner"); // TODO: 4 passes
                        {
                            let mut client_game_max_bid_val = client_game_max_bid_arc.lock().await;
                            *client_game_max_bid_val = Some(msg.max_bid);
                        }
                        {
                            let mut client_game_current_player_val = client_game_current_player_arc.lock().await;
                            *client_game_current_player_val = Some(msg.winner);
                        }
                    }
                    .boxed()
                })
                .on(DUMMY_CARDS_NOTIFICATION, move |payload, _| {
                    let client_dummy_cards_arc = client_dummy_cards_clone.clone();
                    let client_dummy_player_arc = client_dummy_player_clone.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<DummyCardsNotification>(text[0].clone()).unwrap()
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
                .on(ASK_TRICK_NOTIFICATION, move |payload, _| {
                    let client_dummy_cards_arc = client_dummy_cards_clone_1.clone();
                    async move {
                        let msg = match payload {
                            Payload::Text(text) => {
                                serde_json::from_value::<AskTrickNotification>(text[0].clone()).unwrap()
                            }
                            _ => return,
                        };
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
                login_ui(
                    socket.clone(),
                    &runtime,
                    input_nickname.clone()
                );
            }
            GuiClientState::InLobby => {
                list_rooms(
                    socket.clone(),
                    &runtime,
                    client.rooms.clone(),
                    client.state.clone(),
                    input_selected_room_name.clone()
                );
            },
            GuiClientState::CreatingRoom => {
                create_room_ui(
                    socket.clone(),
                    &runtime,
                    input_created_room_name.clone()
                );
            },
            GuiClientState::InRoom => {
                room_ui(
                    socket.clone(),
                    &runtime,
                    client.selected_room_name.clone(),
                    client.seats.clone()
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
                    input_placed_trick_clone.clone(),
                    client.game_current_player.clone(),
                    client.dummy_cards.clone(),
                    client.dummy_player.clone(),
                    &bid_textures,
                    &card_textures,
                );
            }
        }

        display_notifications(client.notifications.clone()).await;

        next_frame().await;
    }
}