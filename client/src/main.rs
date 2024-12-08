use std::{io::Write, sync::Arc, time::Duration};

use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use tokio::{
    sync::{mpsc, Notify},
    time::sleep,
};

use common::{
    message::{
        client_message::{
            JoinRoomMessage, LeaveRoomMessage, ListPlacesMessage, ListRoomsMessage, LoginMessage,
            RegisterRoomMessage, SelectPlaceMessage, JOIN_ROOM_MESSAGE, LEAVE_ROOM_MESSAGE,
            LIST_PLACES_MESSAGE, LIST_ROOMS_MESSAGE, LOGIN_MESSAGE, REGISTER_ROOM_MESSAGE,
            SELECT_PLACE_MESSAGE,
        },
        server_notification::{
            GameStartedNotification, JoinRoomNotification, LeaveRoomNotification,
            SelectPlaceNotification, GAME_STARTED_NOTIFICATION, JOIN_ROOM_NOTIFICATION,
            LEAVE_ROOM_NOTIFICATION, SELECT_PLACE_NOTIFICATION,
        },
        server_response::{
            LeaveRoomResponse, ListPlacesResponse, ListRoomsResponse, LoginResponse,
            SelectPlaceResponse, JOIN_ROOM_RESPONSE, LEAVE_ROOM_RESPONSE, LIST_PLACES_RESPONSE,
            LIST_ROOMS_RESPONSE, LOGIN_RESPONSE, REGISTER_ROOM_RESPONSE, SELECT_PLACE_RESPONSE,
        },
    },
    room::{RoomId, RoomInfo, Visibility},
    user::User,
    Player,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
struct WelcomeMessage {
    username: String,
    // some_welcome_value: i32,
}

#[tokio::main]
async fn main() {
    let game_start_notifier = Arc::new(Notify::new());
    let game_start_notifier_clone = game_start_notifier.clone();

    let register_room_notifier = Arc::new(Notify::new());
    let register_room_notifier_clone = register_room_notifier.clone();

    let (select_username_tx, mut select_username_rx) = mpsc::channel(1);

    let (select_place_tx, mut select_place_rx) = mpsc::channel(1);

    let (room_ids_tx, mut room_ids_rx) = mpsc::channel(1);

    let socket = ClientBuilder::new("http://localhost:3000/")
        .namespace("/")
        .on(LOGIN_RESPONSE, move |payload, s| {
            let select_username_tx = select_username_tx.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<LoginResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                match msg {
                    LoginResponse::Ok => {
                        s.emit(LIST_ROOMS_MESSAGE, to_string(&ListRoomsMessage {}).unwrap())
                            .await
                            .unwrap();
                        select_username_tx.send(true).await.unwrap();
                    }
                    LoginResponse::UsernameAlreadyExists => {
                        println!("Username already exists");
                        select_username_tx.send(false).await.unwrap();
                    }
                    LoginResponse::UserAlreadyLoggedIn => {
                        println!("User is already logged in");
                        select_username_tx.send(false).await.unwrap();
                    }
                }
            }
            .boxed()
        })
        .on(LIST_ROOMS_RESPONSE, move |payload, _| {
            let room_ids_tx = room_ids_tx.clone();
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

                room_ids_tx.send(rooms).await.unwrap();
            }
            .boxed()
        })
        .on(REGISTER_ROOM_RESPONSE, move |_, _| {
            let notifier = register_room_notifier_clone.clone();
            async move {
                // println!("Room registered {:?}", payload);
                notifier.notify_one();
            }
            .boxed()
        })
        .on(JOIN_ROOM_RESPONSE, move |payload, c| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<LoginResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Join room response {:?}", msg);
                c.emit(
                    LIST_PLACES_MESSAGE,
                    to_string(&ListPlacesMessage {}).unwrap(),
                )
                .await
                .unwrap();
            }
            .boxed()
        })
        .on(LIST_PLACES_RESPONSE, move |payload, _| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<ListPlacesResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                match msg {
                    ListPlacesResponse::Ok(msg) => {
                        let positions = msg
                            .into_iter()
                            .map(|user| {
                                if let Some(user) = user {
                                    user.get_username().to_string()
                                } else {
                                    "-".to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" | ");
                        println!("Current positions are | {} |", positions);
                    }
                    ListPlacesResponse::NotInRoom => {
                        println!("[INFO] You are not in a room");
                    }
                    ListPlacesResponse::Unauthenticated => {
                        println!("[INFO] You are not authenticated");
                    }
                }
            }
            .boxed()
        })
        .on(SELECT_PLACE_RESPONSE, move |payload, _| {
            let notify = select_place_tx.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<SelectPlaceResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Select place response {:?}", msg);
                notify.send(msg == SelectPlaceResponse::Ok).await.unwrap();
            }
            .boxed()
        })
        .on(SELECT_PLACE_NOTIFICATION, move |payload, _| {
            async move {
                let player_position = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<SelectPlaceNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Player positions: {:?}", player_position);
            }
            .boxed()
        })
        .on(JOIN_ROOM_NOTIFICATION, move |payload, _| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<JoinRoomNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("User {} joined my room", msg.user.get_username());
            }
            .boxed()
        })
        .on(LEAVE_ROOM_RESPONSE, move |payload, c| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<LeaveRoomResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Leave room response {:?}", msg);
                c.emit(LIST_ROOMS_MESSAGE, to_string(&ListRoomsMessage {}).unwrap())
                    .await
                    .unwrap();
            }
            .boxed()
        })
        .on(LEAVE_ROOM_NOTIFICATION, move |payload, _| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<LeaveRoomNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("User {} left my room", msg.user.get_username());
            }
            .boxed()
        })
        .on(GAME_STARTED_NOTIFICATION, move |payload, _| {
            let notifier = game_start_notifier_clone.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<GameStartedNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Game started {:?}", msg);
                notifier.notify_one();
            }
            .boxed()
        })
        .on("error", |err, _| {
            async move { eprintln!("Error: {:#?}", err) }.boxed()
        })
        .connect()
        .await
        .expect("Connection failed");

    loop {
        let mut username = String::new();

        print!("Enter username: ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut username).unwrap();
        // TODO: filter with regex

        let msg = LoginMessage {
            user: User::new(username.trim()),
        };

        socket
            .emit(LOGIN_MESSAGE, to_string(&msg).unwrap())
            .await
            .unwrap();

        if select_username_rx.recv().await.unwrap() {
            break;
        }
    }

    'lobby_loop: loop {
        let room_ids = room_ids_rx.recv().await.unwrap();

        'room_selection: loop {
            println!("Create new room or join existing:");
            println!("[e] Exit");
            println!("[r] Refresh");
            println!("[0] Create new room");

            for (i, room) in room_ids.iter().enumerate() {
                println!("[{}] Join \"{}\"", i + 1, room);
            }

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            match input.trim() {
                "e" => {
                    break 'lobby_loop;
                }
                "r" => {
                    socket
                        .emit(LIST_ROOMS_MESSAGE, to_string(&ListRoomsMessage {}).unwrap())
                        .await
                        .unwrap();
                    continue 'lobby_loop;
                }
                _ => {}
            }
            let Ok(selection) = input.trim().parse::<usize>() else {
                println!("Invalid selection");
                continue;
            };

            let room_id = if selection == 0 {
                println!("Creating new room");

                let mut room_name = String::new();
                print!("Enter room name: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut room_name).unwrap();
                let room_name = room_name.trim();

                let msg = RegisterRoomMessage {
                    room_info: RoomInfo {
                        id: RoomId::new(room_name),
                        visibility: Visibility::Public,
                    },
                };

                socket
                    .emit(REGISTER_ROOM_MESSAGE, to_string(&msg).unwrap())
                    .await
                    .unwrap();

                register_room_notifier.notified().await;

                room_name.to_string()
            } else if selection - 1 < room_ids.len() {
                room_ids[selection - 1].clone()
            } else {
                println!("Invalid selection");
                continue;
            };

            let msg = JoinRoomMessage {
                room_id: RoomId::new(&room_id),
            };

            println!("Sending join_room {}", room_id);
            socket
                .emit(JOIN_ROOM_MESSAGE, to_string(&msg).unwrap())
                .await
                .unwrap();

            loop {
                print!("Enter position [0-3] Spectator [4] (any other to leave room): ");
                std::io::stdout().flush().unwrap();
                let mut position_string = String::new();
                std::io::stdin().read_line(&mut position_string).unwrap();
                let position = position_string.trim().parse::<i32>().unwrap();

                if position >= 0 && position < 4 {
                    println!("Sending select_place");
                    socket
                        .emit(
                            SELECT_PLACE_MESSAGE,
                            to_string(&SelectPlaceMessage {
                                position: Player::from_usize(position as usize),
                            })
                            .unwrap(),
                        )
                        .await
                        .unwrap();

                    if select_place_rx.recv().await.unwrap() {
                        break 'room_selection;
                    } else {
                        println!("Position already taken");
                        socket
                            .emit(
                                LIST_PLACES_MESSAGE,
                                to_string(&ListPlacesMessage {}).unwrap(),
                            )
                            .await
                            .unwrap();
                    }
                } else if position == 4 {
                    println!("Selected spectator");
                    break 'room_selection;
                } else {
                    println!("Sending leave room");
                    socket
                        .emit(LEAVE_ROOM_MESSAGE, to_string(&LeaveRoomMessage {}).unwrap())
                        .await
                        .unwrap();

                    continue 'lobby_loop;
                }
            }
        }

        game_start_notifier.notified().await;

        println!("Starting game...");

        sleep(Duration::from_secs(2)).await;

        socket
            .emit(LEAVE_ROOM_MESSAGE, to_string(&LeaveRoomMessage {}).unwrap())
            .await
            .unwrap();
    }

    socket.disconnect().await.expect("Disconnect failed");
}
