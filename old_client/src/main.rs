use std::{
    io::Write,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use futures_util::FutureExt;
use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde_json::to_string;
use tokio::sync::{mpsc, Mutex, Notify};

use common::{
    message::{
        client_message::{
            GetCardsMessage, JoinRoomMessage, LeaveRoomMessage, ListPlacesMessage,
            ListRoomsMessage, LoginMessage, MakeBidMessage, MakeTrickMessage, RegisterRoomMessage,
            SelectPlaceMessage,
        },
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
        GetErrorMessage, MessageTrait,
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
    let ask_trick_tx_3 = ask_trick_tx.clone();

    let selected_card = Arc::new(Mutex::new(None));
    let selected_card_clone = selected_card.clone();

    let card_list = Arc::new(Mutex::new(None));
    let card_list_clone = card_list.clone();
    let card_list_clone_2 = card_list.clone();

    let card_list_notify = Arc::new(Notify::new());
    let card_list_notify_clone = card_list_notify.clone();

    let auction_result = Arc::new(Mutex::new(None));
    let auction_result_clone = auction_result.clone();

    let register_room_notifier = Arc::new(Notify::new());
    let register_room_notifier_clone = register_room_notifier.clone();

    let game_finished = Arc::new(AtomicBool::new(false));
    let game_finished_clone = game_finished.clone();

    let (select_username_tx, mut select_username_rx) = mpsc::channel(1);

    let (select_place_tx, mut select_place_rx) = mpsc::channel(1);

    let (room_ids_tx, mut room_ids_rx) = mpsc::channel(1);

    let socket = ClientBuilder::new("http://localhost:3000/")
        .namespace("/")
        .on(LoginResponse::MSG_TYPE, move |payload, s| {
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
                        s.emit(
                            ListRoomsMessage::MSG_TYPE,
                            to_string(&ListRoomsMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();
                        select_username_tx.send(true).await.unwrap();
                    }
                    err => {
                        println!("{}", err.err_msg());
                        select_username_tx.send(false).await.unwrap();
                    }
                }
            }
            .boxed()
        })
        .on(ListRoomsResponse::MSG_TYPE, move |payload, _| {
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
        .on(RegisterRoomResponse::MSG_TYPE, move |_, _| {
            let notifier = register_room_notifier_clone.clone();
            async move {
                // println!("Room registered {:?}", payload);
                notifier.notify_one();
            }
            .boxed()
        })
        .on(JoinRoomResponse::MSG_TYPE, move |payload, c| {
            async move {
                let _msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<LoginResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                c.emit(
                    ListPlacesMessage::MSG_TYPE,
                    to_string(&ListPlacesMessage {}).unwrap(),
                )
                .await
                .unwrap();
            }
            .boxed()
        })
        .on(ListPlacesResponse::MSG_TYPE, move |payload, _| {
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
        .on(SelectPlaceResponse::MSG_TYPE, move |payload, _| {
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
        .on(SelectPlaceNotification::MSG_TYPE, move |payload, _| {
            async move {
                let player_position = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<SelectPlaceNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!(
                    "Player {} selected positions {:?}",
                    player_position.user.get_username(),
                    player_position.position
                );
            }
            .boxed()
        })
        .on(JoinRoomNotification::MSG_TYPE, move |payload, _| {
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
        .on(LeaveRoomResponse::MSG_TYPE, move |payload, c| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<LeaveRoomResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Leave room response {:?}", msg);
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
        .on(GameStartedNotification::MSG_TYPE, move |payload, c| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<GameStartedNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Game started {:?}", msg);
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
            let card_list = card_list_clone.clone();
            let card_list_notify = card_list_notify_clone.clone();
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
                *card_list.lock().await = Some(cards);
                card_list_notify.notify_one();
            }
            .boxed()
        })
        .on(AskBidNotification::MSG_TYPE, move |payload, _| {
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
        .on(MakeBidResponse::MSG_TYPE, move |payload, _| {
            let ask_bid_tx = ask_bid_tx_2.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<MakeBidResponse>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                if let MakeBidResponse::InvalidBid = msg {
                    println!("Invalid bid");
                    ask_bid_tx.send(None).await.unwrap();
                }
            }
            .boxed()
        })
        .on(AuctionFinishedNotification::MSG_TYPE, move |payload, _| {
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
                let msg = match msg {
                    AuctionFinishedNotification::NoWinner => {
                        panic!("No winner");
                    }
                    AuctionFinishedNotification::Winner(m) => m,
                };
                *auction_result.lock().await = Some(msg);
                ask_bid_tx.send(None).await.unwrap();
            }
            .boxed()
        })
        .on(DummyCardsNotification::MSG_TYPE, move |payload, _| {
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<DummyCardsNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!(
                    "Dummy cards {}",
                    msg.cards
                        .iter()
                        .map(Card::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
            .boxed()
        })
        .on(AskTrickNotification::MSG_TYPE, move |payload, _| {
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
        .on(MakeTrickResponse::MSG_TYPE, move |payload, _| {
            let ask_trick_tx = ask_trick_tx_2.clone();
            let card_list = card_list_clone_2.clone();
            let selected_card_clone = selected_card_clone.clone();
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
                    MakeTrickResponse::Ok => {
                        let card = (*selected_card_clone.lock().await).unwrap();
                        card_list
                            .lock()
                            .await
                            .as_mut()
                            .unwrap()
                            .retain(|c| c != &card);
                    }
                    m => {
                        println!("trick response {:?}", m);
                    }
                }
            }
            .boxed()
        })
        .on(TrickFinishedNotification::MSG_TYPE, move |payload, _| {
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
        .on(GameFinishedNotification::MSG_TYPE, move |payload, _| {
            let game_finished_clone = game_finished_clone.clone();
            let ask_trick_tx = ask_trick_tx_3.clone();
            async move {
                let msg = match payload {
                    Payload::Text(text) => {
                        serde_json::from_value::<GameFinishedNotification>(text[0].clone()).unwrap()
                    }
                    _ => return,
                };
                println!("Game finished {:?}", msg);
                game_finished_clone.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
                ask_trick_tx.send(None).await.unwrap();
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
            .emit(LoginMessage::MSG_TYPE, to_string(&msg).unwrap())
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
                        .emit(
                            ListRoomsMessage::MSG_TYPE,
                            to_string(&ListRoomsMessage {}).unwrap(),
                        )
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
                        id: RoomId::new(room_name.into()),
                        visibility: Visibility::Public,
                    },
                };

                socket
                    .emit(RegisterRoomMessage::MSG_TYPE, to_string(&msg).unwrap())
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
                room_id: RoomId::new(room_id.clone().into()),
            };

            socket
                .emit(JoinRoomMessage::MSG_TYPE, to_string(&msg).unwrap())
                .await
                .unwrap();

            loop {
                print!("Enter position [0-3] Spectator [4] (any other to leave room): ");
                std::io::stdout().flush().unwrap();
                let mut position_string = String::new();
                std::io::stdin().read_line(&mut position_string).unwrap();
                let position = position_string.trim().parse::<i32>().unwrap();

                if (0..4).contains(&position) {
                    socket
                        .emit(
                            SelectPlaceMessage::MSG_TYPE,
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
                                ListPlacesMessage::MSG_TYPE,
                                to_string(&ListPlacesMessage {}).unwrap(),
                            )
                            .await
                            .unwrap();
                    }
                } else if position == 4 {
                    println!("Selected spectator");
                    break 'room_selection;
                } else {
                    socket
                        .emit(
                            LeaveRoomMessage::MSG_TYPE,
                            to_string(&LeaveRoomMessage {}).unwrap(),
                        )
                        .await
                        .unwrap();

                    continue 'lobby_loop;
                }
            }
        }

        card_list_notify.notified().await;

        println!("Starting game...");

        println!(
            "Your cards: {}",
            card_list
                .lock()
                .await
                .clone()
                .unwrap()
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
                println!("[d] - Double");
                println!("[r] - Redouble");
                println!("[value] [suit] - Bid");
                println!("Suits are:");
                println!("[C]lubs");
                println!("[D]iamonds");
                println!("[H]earts");
                println!("[S]pades");
                println!("[N]o Trump");

                let mut bid = String::new();
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut bid).unwrap();
                let bid = bid.trim();
                if bid == "p" {
                    socket
                        .emit(
                            MakeBidMessage::MSG_TYPE,
                            to_string(&MakeBidMessage { bid: Bid::Pass }).unwrap(),
                        )
                        .await
                        .unwrap();
                    break;
                } else if bid == "d" {
                    socket
                        .emit(
                            MakeBidMessage::MSG_TYPE,
                            to_string(&MakeBidMessage { bid: Bid::Double }).unwrap(),
                        )
                        .await
                        .unwrap();
                    break;
                } else if bid == "r" {
                    socket
                        .emit(
                            MakeBidMessage::MSG_TYPE,
                            to_string(&MakeBidMessage { bid: Bid::Redouble }).unwrap(),
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
                    "C" => BidType::Trump(Suit::Clubs),
                    "D" => BidType::Trump(Suit::Diamonds),
                    "H" => BidType::Trump(Suit::Hearts),
                    "S" => BidType::Trump(Suit::Spades),
                    "N" => BidType::NoTrump,
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
                        MakeBidMessage::MSG_TYPE,
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

        match auction_result.max_bid {
            Bid::Play(max_bid, typ) => {
                println!(
                    "Auction was won by {:?} with {} {:?}",
                    auction_result.winner, max_bid, typ
                );
            }
            Bid::Pass => {
                println!("Auction was not won");
            }
            Bid::Double | Bid::Redouble => {} // TODO: impossible, remove in the future
        }

        loop {
            let trick = ask_trick_rx.recv().await.unwrap();
            if game_finished.load(Ordering::Relaxed) {
                break 'lobby_loop;
            }
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
                    card_list
                        .lock()
                        .await
                        .clone()
                        .unwrap()
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
                println!("[C]lubs");
                println!("[D]iamonds");
                println!("[H]earts");
                println!("[S]pades");
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

                let Ok(rank) = Rank::from_str(rank) else {
                    println!("Invalid rank");
                    continue;
                };

                let suit = match suit {
                    "C" => Suit::Clubs,
                    "D" => Suit::Diamonds,
                    "H" => Suit::Hearts,
                    "S" => Suit::Spades,
                    _ => {
                        println!("Invalid suit");
                        continue;
                    }
                };

                let card = Card::new(rank, suit);

                selected_card.lock().await.replace(card);

                println!("Playing card {}", card);
                socket
                    .emit(
                        MakeTrickMessage::MSG_TYPE,
                        to_string(&MakeTrickMessage { card }).unwrap(),
                    )
                    .await
                    .unwrap();
                break;
            }
        }
    }

    socket
        .emit(
            LeaveRoomMessage::MSG_TYPE,
            to_string(&LeaveRoomMessage {}).unwrap(),
        )
        .await
        .unwrap();

    socket.disconnect().await.expect("Disconnect failed");
}
