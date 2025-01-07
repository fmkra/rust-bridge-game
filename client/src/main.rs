mod gui_login;
mod gui_error;
mod gui_client;
mod gui_lobby;
mod gui_create_room;

use gui_client::{GuiClientState, GuiClient};
use gui_login::login_ui;
use gui_error::{create_error, display_errors};
use gui_lobby::list_rooms;
use gui_create_room::create_room_ui;

use std::sync::Arc;
use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde_json::to_string;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Mutex};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use common::{
    message::{
        client_message::{
            ListRoomsMessage,
            JoinRoomMessage,
            LIST_ROOMS_MESSAGE,
            JOIN_ROOM_MESSAGE,
            REGISTER_ROOM_MESSAGE,
        },
        server_response::{
            ListRoomsResponse,
            LoginResponse,
            LIST_ROOMS_RESPONSE,
            LOGIN_RESPONSE,
            REGISTER_ROOM_RESPONSE,
        },
    },
    room::RoomId,
};

#[macroquad::main("Bridge card game")]
async fn main() {
    let client = GuiClient::new();
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");
    let nickname = Arc::new(Mutex::new(String::new()));
    let created_room_name = Arc::new(Mutex::new(String::new()));

    let errors_clone = client.errors.clone();
    let name_clone = client.name.clone();
    let state_clone = client.state.clone();
    let state_clone_1 = state_clone.clone();
    let rooms_clone = client.rooms.clone();
    let nickname_clone = nickname.clone();
    let created_room_name_clone = created_room_name.clone();
    let room_name_clone = client.room_name.clone();

    // Connect to the server
    let socket = runtime.block_on(async {
        Arc::new(
            ClientBuilder::new("http://localhost:3000/")
                .namespace("/")
                .on(LOGIN_RESPONSE, move |payload, s| {
                    let errors = errors_clone.clone();
                    let name = name_clone.clone();
                    let state = state_clone.clone();
                    let nickname = nickname_clone.clone();
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

                                s.emit(LIST_ROOMS_MESSAGE, to_string(&ListRoomsMessage {}).unwrap())
                                    .await
                                    .unwrap();
                            }
                            LoginResponse::UsernameAlreadyExists => {
                                println!("Username already exists");
                                create_error(String::from("Username already exists"), errors);
                            }
                            LoginResponse::UserAlreadyLoggedIn => {
                                println!("User is already logged in");
                                create_error(String::from("User is already logged in"), errors);
                            }
                        }
                    }
                    .boxed()
                })
                .on(LIST_ROOMS_RESPONSE, move |payload, _| {
                    let rooms_arc = rooms_clone.clone();
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
                .on(REGISTER_ROOM_RESPONSE, move |_, _| {
                    let state = state_clone_1.clone();
                    let room_name_arc = room_name_clone.clone();
                    let created_room_name_arc = created_room_name_clone.clone();
                    async move {
                        {
                            let mut state_val = state.lock().await;
                            *state_val = GuiClientState::InRoom;
                        }

                        {
                            let mut room_name_val = room_name_arc.lock().await;
                            let mut create_room_name_val = created_room_name_arc.lock().await;
                            *room_name_val = Some(create_room_name_val.clone());
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
                login_ui(socket.clone(), &runtime, nickname.clone());
            }
            GuiClientState::InLobby => {
                list_rooms(socket.clone(), &runtime, client.rooms.clone(), client.state.clone());
            },
            GuiClientState::CreatingRoom => {
                create_room_ui(socket.clone(), &runtime, created_room_name.clone());
            },
            GuiClientState::InRoom => {
                // Handle the InLobby state
            }
            GuiClientState::InSeat => {
                // Handle the InSeat state
            }
            GuiClientState::Playing => {
                // Handle the Playing state
            }
        }

        display_errors(client.errors.clone()).await;

        next_frame().await;
    }
}