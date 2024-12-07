use std::{
    borrow::BorrowMut,
    cell::RefCell,
    io::{Read, Write},
    sync::Arc,
    time::Duration,
};

use futures_util::FutureExt;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Payload,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string};
use tokio::{
    sync::{mpsc, Mutex, Notify},
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

    let (select_place_tx, mut select_place_rx) = mpsc::channel(1);

    let (room_ids_tx, mut room_ids_rx) = mpsc::channel(1);

    // let cb = move |payload, client| {
    //     let notify_clone = notify.clone();
    //     async move {
    //         println!("payload {:?}", payload);
    //         // println!("client {:?}", client.);
    //         notify_clone.notify_one();
    //     }
    //     .boxed()
    // };

    // get a socket that is connected to the admin namespace
    let socket = ClientBuilder::new("http://localhost:3000/")
        .namespace("/")
        // .on("list_rooms", cb)
        // .on("room_registered", move |payload, client| {
        //     async move {
        //         println!("room_registered {:?}", payload);
        //     }
        //     .boxed()
        // })
        // .on_any(move |event, payload, _| {
        //     async move {
        //         println!("Event: {:?}, Payload: {:?}", event, payload);
        //     }
        //     .boxed()
        // })
        .on(LOGIN_RESPONSE, move |payload, s| {
            // let notify = notify_clone.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<LoginResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Login response {:?}", msg);
                s.emit(LIST_ROOMS_MESSAGE, to_string(&ListRoomsMessage {}).unwrap())
                    .await
                    .unwrap();
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

                println!("Create new room or join existing:");
                println!("[0] Create new room");

                for (i, room) in rooms.iter().enumerate() {
                    println!("[{}] Join \"{}\"", i + 1, room);
                }

                room_ids_tx.send(rooms).await.unwrap();
            }
            .boxed()
        })
        .on(REGISTER_ROOM_RESPONSE, move |payload, _| {
            let notifier = register_room_notifier_clone.clone();
            async move {
                println!("Room registered {:?}", payload);
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
                println!("List places response {:?}", msg);
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
        // .on("user_selected_position", move |payload, _| {
        //     async move {
        //         let msg = match payload {
        //             Payload::Text(text) => {
        //                 serde_json::from_value::<UserSelectedPositionMessage>(text[0].clone())
        //                     .unwrap()
        //             }
        //             _ => return,
        //         };
        //         println!(
        //             "User {} selected position {:?}",
        //             msg.user.get_username(),
        //             msg.position
        //         );
        //     }
        //     .boxed()
        // })
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

    let mut username = String::new();

    print!("Enter username: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut username).unwrap();
    // TODO: filter by regex

    let msg = LoginMessage {
        user: User::new(username.trim()),
    };

    println!("Sending login");
    socket
        .emit(LOGIN_MESSAGE, to_string(&msg).unwrap())
        .await
        .unwrap();

    'outer: loop {
        let room_ids = room_ids_rx.recv().await.unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let selection = input.trim().parse::<usize>().unwrap();

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
        } else {
            room_ids[selection - 1].clone()
        };

        let msg = JoinRoomMessage {
            room_id: RoomId::new(&room_id),
        };

        println!("Sending join_room {}", room_id);
        socket
            .emit(JOIN_ROOM_MESSAGE, to_string(&msg).unwrap())
            .await
            .unwrap();

        'inner: loop {
            print!("Enter position [0-3] (any other to leave room): ");
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
                            position: Some(position as usize),
                        })
                        .unwrap(),
                    )
                    .await
                    .unwrap();

                if select_place_rx.recv().await.unwrap() {
                    break 'outer;
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
            } else {
                println!("Sending leave room");
                socket
                    .emit(LEAVE_ROOM_MESSAGE, to_string(&LeaveRoomMessage {}).unwrap())
                    .await
                    .unwrap();
                break 'inner;
            }
        }
    }

    // std::io::stdin().read_to_string(&mut String::new()).unwrap();
    game_start_notifier.notified().await;

    // println!("Sending register_room");
    // socket
    //     .emit(
    //         "register_room",
    //         to_string(&RegisterRoomMessage {
    //             room_info: RoomInfo {
    //                 id: RoomId::new("room1"),
    //                 visibility: Visibility::Public,
    //             },
    //         })
    //         .unwrap(),
    //     )
    //     .await
    //     .unwrap();

    // let msg = Ping {};

    // socket.emit("msg", to_string(&msg).unwrap()).await.unwrap();

    // notify.notified().await;

    println!("Starting game...");

    sleep(Duration::from_secs(2)).await;

    socket.disconnect().await.expect("Disconnect failed");
}
