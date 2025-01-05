use std::sync::Arc;

use common::message::client_message::{
    GetCardsMessage, LeaveRoomMessage, ListPlacesMessage, ListRoomsMessage, MakeBidMessage,
    MakeTrickMessage,
};
use common::message::server_notification::{
    AskBidNotification, AskTrickNotification, AuctionFinishedNotification,
    AuctionFinishedNotificationInner, DummyCardsNotification, GameFinishedNotification,
};
use common::message::server_response::{GetCardsResponse, MakeBidResponse, MakeTrickResponse};
use common::message::{
    client_message::{JoinRoomMessage, LoginMessage, RegisterRoomMessage, SelectPlaceMessage},
    server_notification::{
        GameStartedNotification, JoinRoomNotification, LeaveRoomNotification,
        SelectPlaceNotification,
    },
    server_response::{
        JoinRoomResponse, LeaveRoomResponse, ListPlacesResponse, ListRoomsResponse, LoginResponse,
        RegisterRoomResponse, SelectPlaceResponse,
    },
    MessageTrait,
};
use common::user::User;
use common::{Bid, BidError, BidStatus, GameState, TrickStatus};
use handlers::{notify_trick_finished, RoomWrapper};
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use state::{RoomState, ServerState};

mod handlers;
mod state;

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
            LoginMessage::MSG_TYPE,
            |s: SocketRef, Data::<LoginMessage>(data), state: State<ServerState>| async move {
                // TODO: regex filter username string

                if s.extensions.get::<ClientData>().is_some() {
                    s.emit(LoginResponse::MSG_TYPE, &LoginResponse::UserAlreadyLoggedIn).unwrap();
                }

                if !state.write().await.add_user(data.user.clone()) {
                    s.emit(LoginResponse::MSG_TYPE, &LoginResponse::UsernameAlreadyExists).unwrap();
                    return;
                }

                let client_data = ClientData {
                    user: data.user.clone(),
                    room: None,
                };
                s.extensions.insert(client_data);

                s.emit(LoginResponse::MSG_TYPE, &LoginResponse::Ok).unwrap();

                info!("User \"{}\" logged in", data.user.get_username());
            },
        );

        s.on(
            ListRoomsMessage::MSG_TYPE,
            |s: SocketRef, state: State<ServerState>| async move {
                let rooms = state.read().await.get_room_list().await;
                s.emit(ListRoomsResponse::MSG_TYPE, &ListRoomsResponse { rooms }).unwrap();
            },
        );

        s.on(
            RegisterRoomMessage::MSG_TYPE,
            |s: SocketRef, Data::<RegisterRoomMessage>(data), state: State<ServerState>| async move {
                let Some(client_data) = s.extensions.get::<ClientData>() else {
                    s.emit(RegisterRoomResponse::MSG_TYPE, &RegisterRoomResponse::Unauthenticated).unwrap();
                    return;
                };

                let room_id = data.room_info.id.clone();

                let message = state
                    .write()
                    .await
                    .add_room(data.room_info)
                    .await;

                s.emit(RegisterRoomMessage::MSG_TYPE, &message).unwrap();

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
            JoinRoomMessage::MSG_TYPE,
            |s: SocketRef, Data::<JoinRoomMessage>(data), state: State<ServerState>| async move {
                let Some(mut client_data) = s.extensions.get::<ClientData>() else {
                    s.emit(JoinRoomResponse::MSG_TYPE, &JoinRoomResponse::Unauthenticated).unwrap();
                    return;
                };

                if client_data.room.is_some() {
                    s.emit(JoinRoomResponse::MSG_TYPE, &JoinRoomResponse::AlreadyInRoom).unwrap();
                    return;
                }

                let room_id = data.room_id.clone();

                let Some(room_state) = state.read().await.get_room(&room_id).await else {
                    s.emit(JoinRoomResponse::MSG_TYPE, &JoinRoomResponse::RoomNotFound).unwrap();
                    return;
                };

                room_state.write().await.user_join_room(client_data.user.clone()).await;

                client_data.room = Some(room_state);
                let user = client_data.user.clone();
                s.extensions.insert(client_data);

                let room_wrapper = RoomWrapper(room_id.clone());
                s.join(room_wrapper.clone()).unwrap();

                s.emit(JoinRoomResponse::MSG_TYPE, &JoinRoomResponse::Ok).unwrap();

                info!(
                    "User \"{}\" joined room \"{}\"",
                    user.get_username(),
                    room_id.as_str()
                );

                let msg = JoinRoomNotification { user };
                s.to(room_wrapper).emit(JoinRoomNotification::MSG_TYPE, &msg).unwrap();
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

                s.emit(LeaveRoomResponse::MSG_TYPE, &LeaveRoomResponse::Ok).ok();
            }

            info!("User \"{}\" left room \"{}\"", client_data.user.get_username(), room_id.as_str());

            s.to(RoomWrapper(room_id)).emit(LeaveRoomNotification::MSG_TYPE, &LeaveRoomNotification{user: client_data.user}).ok();
            s.leave_all().ok();
        };

        s.on(LeaveRoomMessage::MSG_TYPE, move |s: SocketRef| async move {
            let Some(client_data) = s.extensions.get::<ClientData>() else {
                s.emit(LeaveRoomMessage::MSG_TYPE, &LeaveRoomResponse::Unauthenticated).unwrap();
                return;
            };
            let Some(room) = client_data.room.clone() else {
                s.emit(LeaveRoomMessage::MSG_TYPE, &LeaveRoomResponse::NotInRoom).unwrap();
                return;
            };

            leave_room_handler(s, client_data, room, true).await;
        });

        s.on(ListPlacesMessage::MSG_TYPE, |s: SocketRef| async move {
            let Some(client_data) = s.extensions.get::<ClientData>() else {
                s.emit(ListPlacesResponse::MSG_TYPE, &ListPlacesResponse::Unauthenticated).unwrap();
                return;
            };
            let Some(room) = client_data.room else {
                s.emit(ListPlacesResponse::MSG_TYPE, &ListPlacesResponse::NotInRoom).unwrap();
                return;
            };

            let player_positions = {
                let room_lock = room.read().await;
                room_lock.get_player_positions()
            };

            s.emit(ListPlacesResponse::MSG_TYPE, &ListPlacesResponse::Ok(player_positions)).unwrap();
        });

        s.on(
            SelectPlaceMessage::MSG_TYPE,
            |s: SocketRef, Data::<SelectPlaceMessage>(data)| async move {
                let Some(client_data) = s.extensions.get::<ClientData>() else {
                    s.emit(SelectPlaceResponse::MSG_TYPE, &SelectPlaceResponse::Unauthenticated).unwrap();
                    return;
                };
                let Some(room) = client_data.room else {
                    s.emit(SelectPlaceResponse::MSG_TYPE, &SelectPlaceResponse::NotInRoom).unwrap();
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
                    s.emit(SelectPlaceResponse::MSG_TYPE, &SelectPlaceResponse::PlaceAlreadyTaken).unwrap();
                    return;
                }

                let position_str = match data.position {
                    Some(pos) => pos.to_u8().to_string(),
                    None => "*spectator*".into(),
                };
                info!("User \"{}\" selected place {} in room \"{}\"", client_data.user.get_username(), position_str, room_id.as_str());

                s.emit(SelectPlaceResponse::MSG_TYPE, &SelectPlaceResponse::Ok).unwrap();

                let msg = SelectPlaceNotification {
                    user: client_data.user,
                    position: data.position,
                };
                s.to(RoomWrapper(room_id.clone())).emit(SelectPlaceNotification::MSG_TYPE, &msg).unwrap();

                if let Some(player_position) = player_position_all_taken {
                    info!("Game started in room \"{}\"", room_id.as_str());

                    let (msg, first_player, previous_game_state) = {
                        let mut room_lock = room.write().await;
                        let previous_game_state = room_lock.game.state.clone();
                        if previous_game_state == GameState::WaitingForPlayers {
                            room_lock.game.start();
                        }
                        let msg = GameStartedNotification {
                            start_position: room_lock.game.current_player,
                            player_position: player_position,
                        };
                        (msg, room_lock.game.current_player, previous_game_state)
                    };

                    let msg2 = AskBidNotification {
                        player: first_player,
                        max_bid: Bid::Pass,
                    };

                    if previous_game_state == GameState::WaitingForPlayers {
                        // Game starts
                        s.within(RoomWrapper(room_id.clone())).emit(GameStartedNotification::MSG_TYPE, &msg).unwrap();

                        s.within(RoomWrapper(room_id.clone())).emit(AskBidNotification::MSG_TYPE, &msg2).unwrap();
                    } else {
                        // Game is already running and is resumed now
                        // TODO: maybe when 2 players leave, let first one in before 2nd joins
                        s.emit(GameStartedNotification::MSG_TYPE, &msg).unwrap();

                        if previous_game_state == GameState::Auction {
                            s.emit(AskBidNotification::MSG_TYPE, &msg2).unwrap();
                        } else {
                            let room_lock = room.read().await;

                            s.emit(AuctionFinishedNotification::MSG_TYPE, &Some(AuctionFinishedNotificationInner {
                                winner: room_lock.game.max_bidder,
                                max_bid: room_lock.game.max_bid,
                                game_value: room_lock.game.game_value,
                            })).unwrap();

                            s.emit(AskTrickNotification::MSG_TYPE, &AskTrickNotification {
                                player: room_lock.game.current_player,
                                cards: room_lock.game.current_trick.clone(),
                            }).unwrap();
                        }
                    }
                }
            }
        );

        s.on(GetCardsMessage::MSG_TYPE, |s: SocketRef| async move {
            let Some(client_data) = s.extensions.get::<ClientData>() else {
                s.emit(GetCardsResponse::MSG_TYPE, &GetCardsResponse::Unauthenticated).unwrap();
                return;
            };
            let Some(room) = client_data.room else {
                s.emit(GetCardsResponse::MSG_TYPE, &GetCardsResponse::NotInRoom).unwrap();
                return;
            };

            let (position, cards) = {
                let room_lock = room.read().await;
                let position = room_lock.find_player_position(&client_data.user);
                let Some(position) = position else {
                    s.emit(GetCardsResponse::MSG_TYPE, &GetCardsResponse::SpectatorNotAllowed).unwrap();
                    return;
                };
                let cards = room_lock.game.get_cards(&position).clone();
                (position, cards)
            };

            let msg = GetCardsResponse::Ok { cards, position };
            s.emit(GetCardsResponse::MSG_TYPE, &msg).unwrap();
        });

        s.on(MakeBidMessage::MSG_TYPE, |s: SocketRef, Data::<MakeBidMessage>(data)| async move {
            let Some(client_data) = s.extensions.get::<ClientData>() else {
                s.emit(MakeBidResponse::MSG_TYPE, &MakeBidResponse::Unauthenticated).unwrap();
                return;
            };
            let Some(room) = client_data.room else {
                s.emit(MakeBidResponse::MSG_TYPE, &MakeBidResponse::NotInRoom).unwrap();
                return;
            };

            let mut room_lock = room.write().await;

            let Some(player) = room_lock.find_player_position(&client_data.user) else {
                s.emit(MakeBidResponse::MSG_TYPE, &MakeBidResponse::SpectatorNotAllowed).unwrap();
                return;
            };

            match room_lock.game.place_bid(&player, data.bid) {
                BidStatus::Error(bid_error) => match bid_error {
                    BidError::GameStateMismatch => {
                        s.emit(MakeBidResponse::MSG_TYPE, &MakeBidResponse::AuctionNotInProcess).unwrap();
                    },
                    BidError::PlayerOutOfTurn => {
                        s.emit(MakeBidResponse::MSG_TYPE, &MakeBidResponse::NotYourTurn).unwrap();
                    },
                    BidError::WrongBid | BidError::CantDouble | BidError::CantRedouble => { // TODO: maybe handle double separately
                        s.emit(MakeBidResponse::MSG_TYPE, &MakeBidResponse::InvalidBid).unwrap();
                    },
                },
                next_state => {
                    s.emit(MakeBidResponse::MSG_TYPE, &MakeBidResponse::Ok).unwrap();

                    let room_handle = RoomWrapper(room_lock.info.id.clone());
                    if next_state == BidStatus::Auction {
                        s.within(room_handle).emit(AskBidNotification::MSG_TYPE, &AskBidNotification {
                            player: room_lock.game.current_player,
                            max_bid: room_lock.game.max_bid,
                        }).unwrap();
                    } else {
                        s.within(room_handle.clone()).emit(AuctionFinishedNotification::MSG_TYPE, &Some(AuctionFinishedNotificationInner {
                            winner: room_lock.game.max_bidder,
                            max_bid: room_lock.game.max_bid,
                            game_value: room_lock.game.game_value,
                        })).unwrap();

                        if next_state == BidStatus::Finished {
                            // 4 passes

                            s.within(room_handle.clone()).emit(GameFinishedNotification::MSG_TYPE, &GameFinishedNotification{result: None}).unwrap();
                        } else {
                            s.within(room_handle).emit(AskTrickNotification::MSG_TYPE, &AskTrickNotification {
                                player: room_lock.game.current_player,
                                cards: room_lock.game.current_trick.clone(),
                            }).unwrap();
                        }
                    }
                },
            }
        });

        s.on(MakeTrickMessage::MSG_TYPE, |s: SocketRef, Data::<MakeTrickMessage>(data)| async move {
            let Some(client_data) = s.extensions.get::<ClientData>() else {
                s.emit(MakeTrickResponse::MSG_TYPE, &MakeTrickResponse::Unauthenticated).unwrap();
                return;
            };
            let Some(room) = client_data.room else {
                s.emit(MakeTrickResponse::MSG_TYPE, &MakeTrickResponse::NotInRoom).unwrap();
                return;
            };

            let mut room_lock = room.write().await;

            let Some(player) = room_lock.find_player_position(&client_data.user) else {
                s.emit(MakeTrickResponse::MSG_TYPE, &MakeTrickResponse::SpectatorNotAllowed).unwrap();
                return;
            };

            let room_id = room_lock.info.id.clone();

            let trick_result = room_lock.game.trick(&player, &data.card);
            s.emit(MakeTrickResponse::MSG_TYPE, &MakeTrickResponse::from(&trick_result)).unwrap();

            match trick_result {
                TrickStatus::Error(_) => {
                    return
                },
                TrickStatus::TrickInProgress => {
                    if room_lock.game.trick_no == 0 && room_lock.game.current_trick.len() == 1 {
                        s.within(RoomWrapper(room_id.clone())).emit(DummyCardsNotification::MSG_TYPE, &DummyCardsNotification::from(room_lock.game.get_dummy().unwrap().clone())).unwrap();
                    }
                }
                TrickStatus::TrickFinished(trick_state) => {
                    notify_trick_finished(&s, &room_id, trick_state);

                }
                TrickStatus::DealFinished(deal_finished) => {
                    notify_trick_finished(&s, &room_id, deal_finished.trick_state);

                    // TODO: send deal finished and game finished notification
                    // if let Some(game_result) = room_lock.game.evaluate() {
                    // s.within(RoomWrapper(room_id.clone())).emit(GAME_FINISHED_NOTIFICATION, &GameFinishedNotification::from(game_result)).unwrap();
                    //     return;
                    // }

                }
            }

            s.within(RoomWrapper(room_id)).emit(AskTrickNotification::MSG_TYPE, &AskTrickNotification {
                player: room_lock.game.current_player,
                cards: room_lock.game.current_trick.clone(),
            }).unwrap();
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
