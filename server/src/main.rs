use std::sync::Arc;
use std::time::Duration;

use common::message::client_message::{
    GetCardsMessage, LeaveRoomMessage, ListPlacesMessage, ListRoomsMessage, MakeBidMessage,
    MakeTrickMessage,
};
use common::message::server_notification::{
    AskBidNotification, AskTrickNotification, AuctionFinishedNotification,
    AuctionFinishedNotificationInner, DealFinishedNotification, DummyCardsNotification,
    GameFinishedNotification, MakeBidNotification, MakeTrickNotification,
    TrickFinishedNotification,
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
use handlers::RoomWrapper;
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use state::{RoomState, ServerState};
use utils::{get_client_or_response, notify, notify_others, send};

mod handlers;
mod state;
mod utils;

#[derive(Clone)]
struct ClientData {
    user: User,
    room: Option<Arc<RwLock<RoomState>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::new();

    tracing::subscriber::set_global_default(subscriber)?;

    let (layer, io) = SocketIo::builder()
        .with_state(ServerState::new(
            RwLock::new(state::ServerStateInner::new()),
        ))
        .build_layer();

    io.ns("/", |s: SocketRef| {
        s.on(
            LoginMessage::MSG_TYPE,
            |s: SocketRef, Data::<LoginMessage>(data), state: State<ServerState>| async move {
                // check username length between 3 and 20
                if !(3..=20).contains(&data.user.get_username().len()) {
                    send(&s, &LoginResponse::UsernameInvalidLength);
                    return;
                }

                // check username characters
                if !data.user.get_username().chars().all(|c| c.is_alphanumeric() || c == '_') {
                    send(&s, &LoginResponse::UsernameInvalidCharacters);
                    return;
                }

                if s.extensions.get::<ClientData>().is_some() {
                    send(&s, &LoginResponse::UserAlreadyLoggedIn);
                }

                if !state.write().await.add_user(data.user.clone()) {
                    send(&s, &LoginResponse::UsernameAlreadyExists);
                    return;
                }

                let client_data = ClientData {
                    user: data.user.clone(),
                    room: None,
                };
                s.extensions.insert(client_data);

                send(&s, &LoginResponse::Ok);

                info!("User \"{}\" logged in", data.user.get_username());
            },
        );

        s.on(
            ListRoomsMessage::MSG_TYPE,
            |s: SocketRef, state: State<ServerState>| async move {
                let rooms = state.read().await.get_room_list().await;
                send(&s, &ListRoomsResponse { rooms });
            },
        );

        s.on(
            RegisterRoomMessage::MSG_TYPE,
            |s: SocketRef, Data::<RegisterRoomMessage>(data), state: State<ServerState>| async move {
                let Some(client_data) = get_client_or_response(&s, &RegisterRoomResponse::Unauthenticated) else {return};

                let room_id = data.room_info.id.clone();

                let message = state
                    .write()
                    .await
                    .add_room(data.room_info)
                    .await;

                send(&s, &message);

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
                let Some(mut client_data) = get_client_or_response(&s, &JoinRoomResponse::Unauthenticated) else {return};

                if client_data.room.is_some() {
                    send(&s, &JoinRoomResponse::AlreadyInRoom);
                    return;
                }

                let room_id = data.room_id.clone();

                let Some(room_state) = state.read().await.get_room(&room_id).await else {
                    send(&s, &JoinRoomResponse::RoomNotFound);
                    return;
                };

                room_state.write().await.user_join_room(client_data.user.clone()).await;

                client_data.room = Some(room_state);
                let user = client_data.user.clone();
                s.extensions.insert(client_data);

                s.join(RoomWrapper(room_id.clone())).unwrap();

                send(&s, &JoinRoomResponse::Ok);

                info!(
                    "User \"{}\" joined room \"{}\"",
                    user.get_username(),
                    room_id.as_str()
                );

                let msg = JoinRoomNotification { user };
                notify_others(&s, &room_id, &msg);
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

                send(&s, &LeaveRoomResponse::Ok);
            }

            info!("User \"{}\" left room \"{}\"", client_data.user.get_username(), room_id.as_str());

            notify_others(&s, &room_id, &LeaveRoomNotification{user: client_data.user});
            s.leave_all().ok();
        };

        s.on(LeaveRoomMessage::MSG_TYPE, move |s: SocketRef| async move {
            let Some(client_data) = get_client_or_response(&s, &LeaveRoomResponse::Unauthenticated) else {return};

            let Some(room) = client_data.room.clone() else {
                send(&s, &LeaveRoomResponse::NotInRoom);
                return;
            };

            leave_room_handler(s, client_data, room, true).await;
        });

        s.on(ListPlacesMessage::MSG_TYPE, |s: SocketRef| async move {
            let Some(client_data) = get_client_or_response(&s, &ListPlacesResponse::Unauthenticated) else {return};

            let Some(room) = client_data.room else {
                send(&s, &ListPlacesResponse::NotInRoom);
                return;
            };

            let player_positions = {
                let room_lock = room.read().await;
                room_lock.get_player_positions()
            };

            send(&s, &ListPlacesResponse::Ok(player_positions));
        });

        s.on(
            SelectPlaceMessage::MSG_TYPE,
            |s: SocketRef, Data::<SelectPlaceMessage>(data)| async move {
                let Some(client_data) = get_client_or_response(&s, &SelectPlaceResponse::Unauthenticated) else {return};

                let Some(room) = client_data.room else {
                    send(&s, &SelectPlaceResponse::NotInRoom);
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
                    send(&s, &SelectPlaceResponse::PlaceAlreadyTaken);
                    return;
                }

                let position_str = match data.position {
                    Some(pos) => pos.to_u8().to_string(),
                    None => "*spectator*".into(),
                };
                info!("User \"{}\" selected place {} in room \"{}\"", client_data.user.get_username(), position_str, room_id.as_str());

                send(&s, &SelectPlaceResponse::Ok);

                notify_others(&s, &room_id, &SelectPlaceNotification {
                    user: client_data.user,
                    position: data.position,
                });

                let game_state = room.read().await.game.state;

                if game_state != GameState::WaitingForPlayers {
                    // Player joined to a game that is already running    
                    room.read().await.send_notifications(&s).await;
                    return;
                }

                if let Some(player_position) = player_position_all_taken {
                    // Game starts now
                    info!("Game started in room \"{}\"", room_id.as_str());

                    let mut room_lock = room.write().await;
                    room_lock.game.start();

                    let notifications = vec![
                        notify(&s, &room_id, GameStartedNotification {
                            start_position: room_lock.game.current_player,
                            player_position: player_position.clone(),
                        }),
                        notify(&s, &room_id, AskBidNotification {
                            player: room_lock.game.current_player,
                            max_bid: Bid::Pass,
                        })
                    ];

                    room_lock.append_notifications(notifications);
                }
            }
        );

        s.on(GetCardsMessage::MSG_TYPE, |s: SocketRef| async move {
            let Some(client_data) = get_client_or_response(&s, &GetCardsResponse::Unauthenticated) else {return};

            let Some(room) = client_data.room else {
                send(&s, &GetCardsResponse::NotInRoom);
                return;
            };

            let (position, cards) = {
                let room_lock = room.read().await;
                let position = room_lock.find_player_position(&client_data.user);
                let Some(position) = position else {
                    send(&s, &GetCardsResponse::SpectatorNotAllowed);
                    return;
                };
                let cards = room_lock.game.get_cards(&position).clone();
                (position, cards)
            };

            let msg = GetCardsResponse::Ok { cards, position };
            send(&s, &msg);
        });

        s.on(MakeBidMessage::MSG_TYPE, |s: SocketRef, Data::<MakeBidMessage>(data)| async move {
            let Some(client_data) = get_client_or_response(&s, &MakeBidResponse::Unauthenticated) else {return};

            let Some(room) = client_data.room else {
                send(&s, &MakeBidResponse::NotInRoom);
                return;
            };

            let mut room_lock = room.write().await;

            let Some(player) = room_lock.find_player_position(&client_data.user) else {
                send(&s, &MakeBidResponse::SpectatorNotAllowed);
                return;
            };

            match room_lock.game.place_bid(&player, data.bid) {
                BidStatus::Error(bid_error) => match bid_error {
                    BidError::GameStateMismatch => {
                        send(&s, &MakeBidResponse::AuctionNotInProcess);
                    },
                    BidError::PlayerOutOfTurn => {
                        send(&s, &MakeBidResponse::NotYourTurn);
                    },
                    BidError::WrongBid | BidError::CantDouble | BidError::CantRedouble => { // TODO: maybe handle double separately
                        send(&s, &MakeBidResponse::InvalidBid);
                    },
                },
                next_state => {
                    send(&s, &MakeBidResponse::Ok);

                    let mut notifications = Vec::new();
                    notifications.push(notify(&s, &room_lock.info.id, MakeBidNotification {
                        player,
                        bid: data.bid,
                    }));
                    if next_state == BidStatus::Auction {
                        notifications.push(notify(&s, &room_lock.info.id, AskBidNotification {
                            player: room_lock.game.current_player,
                            max_bid: room_lock.game.max_bid,
                        }));
                    } else {
                        sleep(Duration::from_secs(2)).await;

                        notifications.push(notify(&s, &room_lock.info.id, AuctionFinishedNotification::Winner(AuctionFinishedNotificationInner {
                            winner: room_lock.game.max_bidder,
                            max_bid: room_lock.game.max_bid,
                            game_value: room_lock.game.game_value,
                        })));

                        if next_state == BidStatus::Finished {
                            // 4 passes

                            notifications.push(notify(&s, &room_lock.info.id, GameFinishedNotification{result: None}));
                        } else {
                            notifications.push(notify(&s, &room_lock.info.id, AskTrickNotification {
                                player: room_lock.game.current_player,
                                cards: room_lock.game.current_trick.clone(),
                            }));
                        }
                    }
                    room_lock.append_notifications(notifications);
                },
            }
        });

        s.on(MakeTrickMessage::MSG_TYPE, |s: SocketRef, Data::<MakeTrickMessage>(data), state: State<ServerState>| async move {
            let Some(client_data) = get_client_or_response(&s, &MakeTrickResponse::Unauthenticated) else {return};

            let Some(room) = client_data.room else {
                send(&s, &MakeTrickResponse::NotInRoom);
                return;
            };

            let mut room_lock = room.write().await;

            let Some(player) = room_lock.find_player_position(&client_data.user) else {
                send(&s, &MakeTrickResponse::SpectatorNotAllowed);
                return;
            };

            let room_id = room_lock.info.id.clone();

            let trick_result = room_lock.game.trick(&player, &data.card);
            send(&s, &MakeTrickResponse::from(&trick_result));

            let mut notifications = Vec::new();

            if let TrickStatus::Error(_) = trick_result {
                return;
            }

            notifications.push(notify(&s, &room_id, MakeTrickNotification {
                player,
                card: data.card,
            }));

            match trick_result {
                TrickStatus::TrickInProgress => {
                    if room_lock.game.trick_no == 0 && room_lock.game.current_trick.len() == 1 {
                        let msg = DummyCardsNotification::new(room_lock.game.get_dummy_cards().unwrap().clone(),
                        room_lock.game.get_dummy_player().unwrap());
                        notifications.push(notify(&s, &room_id, msg));
                    }
                }
                TrickStatus::TrickFinished(trick_state) => {
                    sleep(Duration::from_secs(2)).await;

                    notifications.push(notify(&s, &room_id, TrickFinishedNotification::from(trick_state)));
                }
                TrickStatus::DealFinished(deal_finished) => {
                    sleep(Duration::from_secs(2)).await;

                    notifications.push(notify(&s, &room_id, TrickFinishedNotification::from(deal_finished.trick_state.clone())));

                    sleep(Duration::from_secs(2)).await;

                    notifications.push(notify(&s, &room_id, DealFinishedNotification::from(deal_finished.clone())));

                    if deal_finished.is_game_finished {
                        notifications.push(notify(&s, &room_id, GameFinishedNotification{result: None}));

                        state.write().await.remove_room(&room_id);

                        return;
                    } else {
                        room_lock.game.start();

                        notifications.push(notify(&s, &room_id, AskBidNotification {
                            player: deal_finished.next_deal_bidder,
                            max_bid: Bid::Pass,
                        }));
                    }
                }
                TrickStatus::Error(_) => ()
            }

            notifications.push(notify(&s, &room_id, AskTrickNotification {
                player: room_lock.game.current_player,
                cards: room_lock.game.current_trick.clone(),
            }));

            room_lock.append_notifications(notifications);
        });

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
    let args = clap::Command::new("bridge-server")
        .arg(
            clap::Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to listen on")
                .default_value("3000"),
        )
        .get_matches();

    let port = args.get_one::<String>("port").unwrap();
    let addr = format!("0.0.0.0:{}", port);

    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
