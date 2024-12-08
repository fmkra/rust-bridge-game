use std::borrow::Cow;
use std::sync::Arc;

use common::message::client_message::GET_CARDS_MESSAGE;
use common::message::server_response::{GetCardsResponse, GET_CARDS_RESPONSE};
use common::user::User;
use common::{
    message::{
        client_message::{
            JoinRoomMessage, LoginMessage, RegisterRoomMessage, SelectPlaceMessage,
            JOIN_ROOM_MESSAGE, LEAVE_ROOM_MESSAGE, LIST_PLACES_MESSAGE, LIST_ROOMS_MESSAGE,
            LOGIN_MESSAGE, REGISTER_ROOM_MESSAGE, SELECT_PLACE_MESSAGE,
        },
        server_notification::{
            GameStartedNotification, JoinRoomNotification, LeaveRoomNotification,
            SelectPlaceNotification, GAME_STARTED_NOTIFICATION, JOIN_ROOM_NOTIFICATION,
            LEAVE_ROOM_NOTIFICATION, SELECT_PLACE_NOTIFICATION,
        },
        server_response::{
            JoinRoomResponse, LeaveRoomResponse, ListPlacesResponse, ListRoomsResponse,
            LoginResponse, RegisterRoomResponse, SelectPlaceResponse, JOIN_ROOM_RESPONSE,
            LEAVE_ROOM_RESPONSE, LIST_PLACES_RESPONSE, LIST_ROOMS_RESPONSE, LOGIN_RESPONSE,
            REGISTER_ROOM_RESPONSE, SELECT_PLACE_RESPONSE,
        },
    },
    Player,
};
use socketioxide::{
    adapter::Room as SRoom,
    extract::{Data, SocketRef, State},
    operators::RoomParam,
    SocketIo,
};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use common::room::RoomId;
use state::{RoomState, ServerState};

mod state;

#[derive(Clone)]
struct RoomWrapper(RoomId);
// TODO: move it somewhere else
impl RoomParam for RoomWrapper {
    type IntoIter = std::iter::Once<SRoom>;
    #[inline(always)]
    fn into_room_iter(self) -> Self::IntoIter {
        std::iter::once(Cow::Owned(self.0.as_str().into()))
    }
}

#[derive(Debug, Clone)]
struct ClientData {
    user: User,
    room: Option<Arc<RwLock<RoomState>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::new();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting server");

    let (layer, io) = SocketIo::builder()
        .with_state(ServerState::new(
            RwLock::new(state::ServerStateInner::new()),
        ))
        .build_layer();

    io.ns("/", |s: SocketRef| {
        s.on(
            LOGIN_MESSAGE,
            |s: SocketRef, Data::<LoginMessage>(data), state: State<ServerState>| async move {
                // TODO: regex filter username string

                if s.extensions.get::<ClientData>().is_some() {
                    s.emit(LOGIN_RESPONSE, &LoginResponse::UserAlreadyLoggedIn).unwrap();
                }

                if !state.write().await.add_user(data.user.clone()) {
                    s.emit(LOGIN_RESPONSE, &LoginResponse::UsernameAlreadyExists).unwrap();
                    return;
                }

                let client_data = ClientData {
                    user: data.user.clone(),
                    room: None,
                };
                s.extensions.insert(client_data);

                s.emit(LOGIN_RESPONSE, &LoginResponse::Ok).unwrap();

                info!("User \"{}\" logged in", data.user.get_username());
            },
        );

        s.on(
            LIST_ROOMS_MESSAGE,
            |s: SocketRef, state: State<ServerState>| async move {
                let rooms = state.read().await.get_room_list().await;
                s.emit(LIST_ROOMS_RESPONSE, &ListRoomsResponse { rooms }).unwrap();
            },
        );

        s.on(
            REGISTER_ROOM_MESSAGE,
            |s: SocketRef, Data::<RegisterRoomMessage>(data), state: State<ServerState>| async move {
                let Some(client_data) = s.extensions.get::<ClientData>() else {
                    s.emit(REGISTER_ROOM_RESPONSE, &RegisterRoomResponse::Unauthenticated).unwrap();
                    return;
                };

                let room_id = data.room_info.id.clone();

                let message = state
                    .write()
                    .await
                    .add_room(data.room_info)
                    .await;

                s.emit(REGISTER_ROOM_RESPONSE, &message).unwrap();

                if message == RegisterRoomResponse::Ok {
                    info!(
                        "Room \"{}\" was registered by \"{}\"",
                        room_id.as_str(),
                        client_data.user.get_username()
                    );
                }
            },
        );

        s.on(
            JOIN_ROOM_MESSAGE,
            |s: SocketRef, Data::<JoinRoomMessage>(data), state: State<ServerState>| async move {
                let Some(mut client_data) = s.extensions.get::<ClientData>() else {
                    s.emit(JOIN_ROOM_RESPONSE, &JoinRoomResponse::Unauthenticated).unwrap();
                    return;
                };

                if client_data.room.is_some() {
                    s.emit(JOIN_ROOM_RESPONSE, &JoinRoomResponse::AlreadyInRoom).unwrap();
                    return;
                }

                let room_id = data.room_id.clone();

                let Some(room_state) = state.read().await.get_room(&room_id).await else {
                    s.emit(JOIN_ROOM_RESPONSE, &JoinRoomResponse::RoomNotFound).unwrap();
                    return;
                };

                room_state.write().await.user_join_room(client_data.user.clone()).await;

                client_data.room = Some(room_state);
                let user = client_data.user.clone();
                s.extensions.insert(client_data);

                let room_wrapper = RoomWrapper(room_id.clone());
                s.join(room_wrapper.clone()).unwrap();

                s.emit(JOIN_ROOM_RESPONSE, &JoinRoomResponse::Ok).unwrap();

                info!(
                    "User \"{}\" joined room \"{}\"",
                    user.get_username(),
                    room_id.as_str()
                );

                let msg = JoinRoomNotification { user };
                s.to(room_wrapper).emit(JOIN_ROOM_NOTIFICATION, &msg).unwrap();
            },
        );

        let leave_room_handler = |s: SocketRef, mut client_data: ClientData, room: Arc<RwLock<RoomState>>, generate_response: bool| async move {
            let room_id = {
                let mut room_lock = room.write().await;
                let room_id = room_lock.info.id.clone();
                room_lock.user_leave_room(&client_data.user);
                room_id
            };

            if generate_response {
                client_data.room = None;
                s.extensions.insert(client_data.clone());

                s.emit(LEAVE_ROOM_RESPONSE, &LeaveRoomResponse::Ok).unwrap();
            }

            info!("User \"{}\" left room \"{}\"", client_data.user.get_username(), room_id.as_str());

            s.to(RoomWrapper(room_id)).emit(LEAVE_ROOM_NOTIFICATION, &LeaveRoomNotification{user: client_data.user}).unwrap();
            s.leave_all().unwrap();
        };

        s.on(LEAVE_ROOM_MESSAGE, move |s: SocketRef| async move {
            let Some(client_data) = s.extensions.get::<ClientData>() else {
                s.emit(LEAVE_ROOM_RESPONSE, &LeaveRoomResponse::Unauthenticated).unwrap();
                return;
            };
            let Some(room) = client_data.room.clone() else {
                s.emit(LEAVE_ROOM_RESPONSE, &LeaveRoomResponse::NotInRoom).unwrap();
                return;
            };

            leave_room_handler(s, client_data, room, true).await;
        });

        s.on(LIST_PLACES_MESSAGE, |s: SocketRef| async move {
            let Some(client_data) = s.extensions.get::<ClientData>() else {
                s.emit(LIST_PLACES_RESPONSE, &ListPlacesResponse::Unauthenticated).unwrap();
                return;
            };
            let Some(room) = client_data.room else {
                s.emit(LIST_PLACES_RESPONSE, &ListPlacesResponse::NotInRoom).unwrap();
                return;
            };

            let player_positions = {
                let room_lock = room.read().await;
                room_lock.get_player_positions()
            };

            s.emit(LIST_PLACES_RESPONSE, &ListPlacesResponse::Ok(player_positions)).unwrap();
        });

        s.on(
            SELECT_PLACE_MESSAGE,
            |s: SocketRef, Data::<SelectPlaceMessage>(data)| async move {
                let Some(client_data) = s.extensions.get::<ClientData>() else {
                    s.emit(SELECT_PLACE_RESPONSE, &SelectPlaceResponse::Unauthenticated).unwrap();
                    return;
                };
                let Some(room) = client_data.room else {
                    s.emit(SELECT_PLACE_RESPONSE, &SelectPlaceResponse::NotInRoom).unwrap();
                    return;
                };

                let (is_place_free, player_positions, room_id) = {
                    let mut room_state_lock = room.write().await;
                    (
                        room_state_lock.user_select_place(&client_data.user, data.position),
                        room_state_lock.get_player_positions(),
                        room_state_lock.info.id.clone()
                    )
                };

                let player_position_all_taken: Option<[User; 4]> = player_positions
                    .clone()
                    .into_iter()
                    .collect::<Option<Vec<User>>>()
                    .map(|v| {
                        v.try_into().unwrap()
                    });

                if !is_place_free {
                    s.emit(SELECT_PLACE_RESPONSE, &SelectPlaceResponse::PlaceAlreadyTaken).unwrap();
                    return;
                }

                let position_str = match data.position {
                    Some(pos) => pos.to_u8().to_string(),
                    None => "*spectator*".into(),
                };
                info!("User \"{}\" selected place {} in room \"{}\"", client_data.user.get_username(), position_str, room_id.as_str());

                s.emit(SELECT_PLACE_RESPONSE, &SelectPlaceResponse::Ok).unwrap();

                let msg = SelectPlaceNotification {
                    user: client_data.user,
                    position: data.position,
                };
                s.to(RoomWrapper(room_id.clone())).emit(SELECT_PLACE_NOTIFICATION, &msg).unwrap();

                if let Some(player_position) = player_position_all_taken {
                    info!("Game started in room \"{}\"", room_id.as_str());

                    let msg = {
                        let mut room_lock = room.write().await;
                        room_lock.game.start();
                        GameStartedNotification {
                            start_position: room_lock.game.current_player,
                            player_position: player_position,
                        }
                    };

                    s.within(RoomWrapper(room_id.clone()))
                        .emit(GAME_STARTED_NOTIFICATION, &msg)
                        .unwrap();
                }
            }
        );

        s.on(GET_CARDS_MESSAGE, |s: SocketRef| async move {
            let Some(client_data) = s.extensions.get::<ClientData>() else {
                s.emit(GET_CARDS_RESPONSE, &GetCardsResponse::Unauthenticated).unwrap();
                return;
            };
            let Some(room) = client_data.room else {
                s.emit(GET_CARDS_RESPONSE, &GetCardsResponse::NotInRoom).unwrap();
                return;
            };

            let cards = {
                let room_lock = room.read().await;
                let player = room_lock.find_player_position(&client_data.user);
                room_lock.game.get_cards(&player.unwrap()).clone() // TODO: handle spectator user
            };

            s.emit(GET_CARDS_RESPONSE, &GetCardsResponse::Ok(cards)).unwrap();
        });

        // s.on(
        //     "msg",
        //     |s: SocketRef, Data::<Ping>(data), state: State<ServerState>| async move {
        //         println!("data = {:?}", data);
        //         println!("state = {:?}", state.lock().await);
        //         println!("rooms = {:?}", s.rooms());
        //     },
        // );

        s.on_disconnect(
            move |s: SocketRef, state: State<ServerState>| async move {
                let Some(client_data) = s.extensions.get::<ClientData>() else { return; };

                let username = client_data.user.get_username();

                state.write().await.remove_user(&client_data.user);

                if let Some(room) = client_data.room.clone() {
                    leave_room_handler(s, client_data.clone(), room, false).await;
                }

                info!("User \"{}\" disconnected", username);
            },
        );
    });

    let app = axum::Router::new()
        .nest_service("/", ServeDir::new("dist"))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(layer),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
