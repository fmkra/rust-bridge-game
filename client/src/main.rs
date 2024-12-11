use std::{io::Write, sync::Arc, time::Duration};

use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use tokio::{
    sync::{mpsc, Mutex, Notify},
    time::sleep,
};

use common::{
    card,
    message::{
        client_message::{
            GetCardsMessage, JoinRoomMessage, LeaveRoomMessage, ListPlacesMessage,
            ListRoomsMessage, LoginMessage, MakeBidMessage, MakeTrickMessage, RegisterRoomMessage,
            SelectPlaceMessage, GET_CARDS_MESSAGE, JOIN_ROOM_MESSAGE, LEAVE_ROOM_MESSAGE,
            LIST_PLACES_MESSAGE, LIST_ROOMS_MESSAGE, LOGIN_MESSAGE, MAKE_BID_MESSAGE,
            MAKE_TRICK_MESSAGE, REGISTER_ROOM_MESSAGE, SELECT_PLACE_MESSAGE,
        },
        server_notification::{
            AskBidNotification, AskTrickNotification, AuctionFinishedNotification,
            AuctionFinishedNotificationInner, GameStartedNotification, JoinRoomNotification,
            LeaveRoomNotification, SelectPlaceNotification, TrickFinishedNotification,
            ASK_BID_NOTIFICATION, ASK_TRICK_NOTIFICATION, AUCTION_FINISHED_NOTIFICATION,
            GAME_STARTED_NOTIFICATION, JOIN_ROOM_NOTIFICATION, LEAVE_ROOM_NOTIFICATION,
            SELECT_PLACE_NOTIFICATION, TRICK_FINISHED_NOTIFICATION,
        },
        server_response::{
            GetCardsResponse, LeaveRoomResponse, ListPlacesResponse, ListRoomsResponse,
            LoginResponse, MakeBidResponse, MakeTrickResponse, SelectPlaceResponse,
            GET_CARDS_RESPONSE, JOIN_ROOM_RESPONSE, LEAVE_ROOM_RESPONSE, LIST_PLACES_RESPONSE,
            LIST_ROOMS_RESPONSE, LOGIN_RESPONSE, MAKE_BID_RESPONSE, MAKE_TRICK_RESPONSE,
            REGISTER_ROOM_RESPONSE, SELECT_PLACE_RESPONSE,
        },
    },
    room::{RoomId, RoomInfo, Visibility},
    user::User,
    Bid, BidType, Card, Player, Rank, Suit,
};

#[tokio::main]
async fn main() {
    let (my_position_tx, mut my_position_rx) = mpsc::channel(1);
    let my_position_tx_1 = my_position_tx.clone();

    let (ask_bid_tx, mut ask_bid_rx) = mpsc::channel(1);
    let ask_bid_tx_1 = ask_bid_tx.clone();
    let ask_bid_tx_2 = ask_bid_tx.clone();
    let ask_bid_tx_3 = ask_bid_tx.clone();

    let (ask_trick_tx, mut ask_trick_rx) = mpsc::channel(1);
    let ask_trick_tx_1 = ask_trick_tx.clone();
    let ask_trick_tx_2 = ask_trick_tx.clone();

    let (card_list_tx, mut card_list_rx) = mpsc::channel(1);
    let card_list_tx_1 = card_list_tx.clone();

    let auction_result = Arc::new(Mutex::new(None));
    let auction_result_clone = auction_result.clone();

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
        .on(GAME_STARTED_NOTIFICATION, move |payload, c| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<GameStartedNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Game started {:?}", msg);
                c.emit(GET_CARDS_MESSAGE, to_string(&GetCardsMessage {}).unwrap())
                    .await
                    .unwrap();
            }
            .boxed()
        })
        .on(GET_CARDS_RESPONSE, move |payload, _| {
            let card_list_tx = card_list_tx_1.clone();
            let my_position_tx = my_position_tx_1.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<GetCardsResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                let cards = match msg {
                    GetCardsResponse::Ok { cards, position } => {
                        my_position_tx.send(position).await.unwrap();
                        cards
                    }
                    _ => {
                        return;
                    }
                };
                card_list_tx.send(cards).await.unwrap();
            }
            .boxed()
        })
        .on(ASK_BID_NOTIFICATION, move |payload, _| {
            let ask_bid_tx = ask_bid_tx_1.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<AskBidNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                ask_bid_tx.send(Some(msg)).await.unwrap();
            }
            .boxed()
        })
        .on(MAKE_BID_RESPONSE, move |payload, _| {
            let ask_bid_tx = ask_bid_tx_2.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<MakeBidResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                match msg {
                    MakeBidResponse::InvalidBid => {
                        println!("Invalid bid");
                        ask_bid_tx.send(None).await.unwrap();
                    }
                    _ => {}
                }
            }
            .boxed()
        })
        .on(AUCTION_FINISHED_NOTIFICATION, move |payload, _| {
            let auction_result = auction_result_clone.clone();
            let ask_bid_tx = ask_bid_tx_3.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<AuctionFinishedNotification>(text[0].clone())
                            .unwrap()
                    }
                    _ => return,
                };
                let msg = msg.expect("No winner"); // TODO: 4 passes
                *auction_result.lock().await = Some(msg);
                ask_bid_tx.send(None).await.unwrap();
            }
            .boxed()
        })
        .on(ASK_TRICK_NOTIFICATION, move |payload, _| {
            let ask_trick_tx = ask_trick_tx_1.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<AskTrickNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                ask_trick_tx.send(Some(msg)).await.unwrap();
            }
            .boxed()
        })
        .on(MAKE_TRICK_RESPONSE, move |payload, _| {
            let ask_trick_tx = ask_trick_tx_2.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<MakeTrickResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                match msg {
                    MakeTrickResponse::InvalidCard => {
                        println!("Invalid card");
                        ask_trick_tx.send(None).await.unwrap();
                    }
                    m => {
                        println!("trick response {:?}", m);
                    }
                }
            }
            .boxed()
        })
        .on(TRICK_FINISHED_NOTIFICATION, move |payload, _| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<TrickFinishedNotification>(text[0].clone())
                            .unwrap()
                    }
                    _ => return,
                };
                println!(
                    "Trick {} taken by {:?}",
                    msg.cards
                        .iter()
                        .map(Card::to_string)
                        .collect::<Vec<_>>()
                        .join(" "),
                    msg.taker
                );
            }
            .boxed()
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

        let cards = card_list_rx.recv().await.unwrap();

        println!("Starting game...");

        println!(
            "Your cards: {}",
            cards
                .iter()
                .map(Card::to_string)
                .collect::<Vec<String>>()
                .join(" ")
        );

        let my_position = my_position_rx.recv().await.unwrap();
        let mut persistent_bid = None;
        let auction_result = loop {
            let new_bid = ask_bid_rx.recv().await.unwrap();
            if let Some(b) = new_bid {
                persistent_bid = Some(b);
            } else {
                let mut mutex = auction_result.lock().await;
                if let Some(result) = mutex.as_ref() {
                    let r = result.clone();
                    *mutex = None;
                    break r;
                }
            }
            let bid = persistent_bid.clone().unwrap();

            if bid.player != my_position {
                println!("{:?} is bidding", bid.player);
                continue;
            }
            println!("Your turn to bid.");
            println!("Highest bid is {}", bid.max_bid.to_str());

            loop {
                println!("[p] - Pass");
                println!("[value] [suit] - Bid");
                println!("Suits are:");
                println!("0 - Clubs");
                println!("1 - Diamonds");
                println!("2 - Hearts");
                println!("3 - Spades");
                println!("4 - No Trump");

                let mut bid = String::new();
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut bid).unwrap();
                let bid = bid.trim();
                if bid == "p" {
                    socket
                        .emit(
                            MAKE_BID_MESSAGE,
                            to_string(&MakeBidMessage { bid: Bid::Pass }).unwrap(),
                        )
                        .await
                        .unwrap();
                    break;
                }
                let Some((value, trump)) = bid.split_once(" ") else {
                    println!("Invalid bid");
                    continue;
                };
                let Ok(value) = value.parse::<u8>() else {
                    println!("Invalid value");
                    continue;
                };
                let trump = match trump {
                    "0" => BidType::Trump(Suit::Clubs),
                    "1" => BidType::Trump(Suit::Diamonds),
                    "2" => BidType::Trump(Suit::Hearts),
                    "3" => BidType::Trump(Suit::Spades),
                    "4" => BidType::NoTrump,
                    _ => {
                        println!("Invalid suit");
                        continue;
                    }
                };
                let Some(bid) = Bid::new(value, trump) else {
                    println!("Invalid value");
                    continue;
                };

                socket
                    .emit(
                        MAKE_BID_MESSAGE,
                        to_string(&MakeBidMessage { bid }).unwrap(),
                    )
                    .await
                    .unwrap();
                // if let Ok(card) = Card::from(
                //     Suit::from(card.chars().nth(0).unwrap()),
                //     card[1..].parse::<u8>().unwrap(),
                // ) {
                //     break;
                // } else {
                //     println!("Invalid card");
                // }
                break;
            }
        };

        let mut persistent_trick = None;

        println!("Auction result is {:?}", auction_result);

        loop {
            let trick = ask_trick_rx.recv().await.unwrap();
            if let Some(t) = trick {
                persistent_trick = Some(t);
            }

            let trick = persistent_trick.clone().unwrap();

            if trick.player != my_position {
                println!("{:?} is tricking", trick.player);
                continue;
            }

            loop {
                println!(
                    "Your cards: {}",
                    cards
                        .iter()
                        .map(Card::to_string)
                        .collect::<Vec<String>>()
                        .join(" ")
                );

                let mut trick_string = trick
                    .cards
                    .iter()
                    .map(Card::to_string)
                    .collect::<Vec<String>>()
                    .join(" ");
                if trick_string.is_empty() {
                    trick_string.push_str("[empty]");
                }

                println!("Trick to: {}", trick_string);

                println!("[rank] [suit]");
                println!("Suits are:");
                println!("0 - Clubs");
                println!("1 - Diamonds");
                println!("2 - Hearts");
                println!("3 - Spades");
                println!("Ranks are:");
                println!("2-10 | J | Q | K | A");

                let mut card = String::new();
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut card).unwrap();
                let card = card.trim();

                let Some((rank, suit)) = card.split_once(" ") else {
                    println!("Invalid card");
                    continue;
                };

                let Some(rank) = Rank::from_str(rank) else {
                    println!("Invalid rank");
                    continue;
                };

                let suit = match suit {
                    "0" => Suit::Clubs,
                    "1" => Suit::Diamonds,
                    "2" => Suit::Hearts,
                    "3" => Suit::Spades,
                    _ => {
                        println!("Invalid suit");
                        continue;
                    }
                };

                let card = Card::new(rank, suit);

                println!("Playing card {}", card.to_string());
                socket
                    .emit(
                        MAKE_TRICK_MESSAGE,
                        to_string(&MakeTrickMessage { card }).unwrap(),
                    )
                    .await
                    .unwrap();
                break;
                // if let Ok(card) = Card::from(
                //     Suit::from(card.chars().nth(0).unwrap()),
                //     card[1..].parse::<u8>().unwrap(),
                // ) {
                //     break;
                // } else {
                //     println!("Invalid card");
                // }
            }
        }

        socket
            .emit(LEAVE_ROOM_MESSAGE, to_string(&LeaveRoomMessage {}).unwrap())
            .await
            .unwrap();

        // println!("=waiting on lock=");
        // my_position_ready.notified().await;
        // println!("=lock released=");
        // if msg.player == my_position.lock().await.unwrap() {
        //     println!(
        //         "It's your turn\nTrick is: {}",
        //         msg.trick_cards
        //             .iter()
        //             .map(card::Card::to_string)
        //             .collect::<Vec<String>>()
        //             .join(" ")
        //     );
        //     turn_notifier.notify_one();
        // }
    }

    socket.disconnect().await.expect("Disconnect failed");
}
