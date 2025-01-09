use serde::{Deserialize, Serialize};

use crate::{
    room::{RoomId, RoomInfo},
    user::User,
};

pub trait MessageTrait {
    const MSG_TYPE: &'static str;
}

/// Messages sent from client to server
pub mod client_message {
    use super::*;
    use crate::{Bid, Card, Player};

    /// Message sent by client when attempting to login
    /// Server answers with LoginResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct LoginMessage {
        pub user: User,
    }
    impl MessageTrait for LoginMessage {
        const MSG_TYPE: &'static str = "login";
    }

    /// Message sent by client when requesting list of public rooms
    /// Server answers with ListRoomsResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ListRoomsMessage {}
    impl MessageTrait for ListRoomsMessage {
        const MSG_TYPE: &'static str = "list_rooms";
    }

    /// Message sent by client when attempting to register a new room
    /// Server answers with RegisterRoomResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RegisterRoomMessage {
        pub room_info: RoomInfo,
    }
    impl MessageTrait for RegisterRoomMessage {
        const MSG_TYPE: &'static str = "register_room";
    }

    /// Message sent by client when attempting to join a room
    /// Server answers with JoinRoomResponse message
    /// Server sends JoinRoomNotification to all users in the room
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct JoinRoomMessage {
        pub room_id: RoomId,
    }
    impl MessageTrait for JoinRoomMessage {
        const MSG_TYPE: &'static str = "join_room";
    }

    /// Message sent by client when attempting to leave a room
    /// Server answers with LeaveRoomResponse message
    /// Server sends LeaveRoomNotification to all users in the room
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct LeaveRoomMessage {}
    impl MessageTrait for LeaveRoomMessage {
        const MSG_TYPE: &'static str = "leave_room";
    }

    /// Message sent by client when requesting list of places in the room
    /// Server answers with ListPlacesResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ListPlacesMessage {}
    impl MessageTrait for ListPlacesMessage {
        const MSG_TYPE: &'static str = "list_places";
    }

    /// Message sent by client when selecting a place in the room
    /// Server answers with UserSelectedPositionMessage message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct SelectPlaceMessage {
        pub position: Option<Player>,
    }
    impl MessageTrait for SelectPlaceMessage {
        const MSG_TYPE: &'static str = "select_place";
    }

    /// Message sent by client when requesting his list of cards
    /// Server answers with GetCardsResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GetCardsMessage {}
    impl MessageTrait for GetCardsMessage {
        const MSG_TYPE: &'static str = "get_cards";
    }

    /// Message sent by client when making a bid
    /// Server answers with MakeBidResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MakeBidMessage {
        pub bid: Bid,
    }
    impl MessageTrait for MakeBidMessage {
        const MSG_TYPE: &'static str = "make_bid";
    }

    /// Message sent by client when making a trick
    /// Server answers with MakeTrickResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MakeTrickMessage {
        pub card: Card,
    }
    impl MessageTrait for MakeTrickMessage {
        const MSG_TYPE: &'static str = "make_trick";
    }
}

pub mod server_response {
    use super::*;
    use crate::{Card, Player, TrickError, TrickStatus};

    /// Answer from server for LoginMessage
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum LoginResponse {
        Ok,
        UsernameAlreadyExists,
        UserAlreadyLoggedIn,
    }
    impl MessageTrait for LoginResponse {
        const MSG_TYPE: &'static str = "login_response";
    }

    /// Answer from server for ListRoomsMessage
    /// Returns list of ids of public rooms
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ListRoomsResponse {
        pub rooms: Vec<RoomId>,
    }
    impl MessageTrait for ListRoomsResponse {
        const MSG_TYPE: &'static str = "list_rooms_response";
    }

    /// Answer from server for RegisterRoomMessage
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub enum RegisterRoomResponse {
        Ok,
        RoomIdAlreadyExists,
        Unauthenticated,
    }
    impl MessageTrait for RegisterRoomResponse {
        const MSG_TYPE: &'static str = "register_room_response";
    }

    /// Answer from server for JoinRoomMessage
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum JoinRoomResponse {
        Ok,
        AlreadyInRoom,
        RoomNotFound,
        Unauthenticated,
    }
    impl MessageTrait for JoinRoomResponse {
        const MSG_TYPE: &'static str = "join_room_response";
    }

    /// Answer from server for LeaveRoomMessage
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum LeaveRoomResponse {
        Ok,
        NotInRoom,
        Unauthenticated,
    }
    impl MessageTrait for LeaveRoomResponse {
        const MSG_TYPE: &'static str = "leave_room_response";
    }

    /// Answer from server for ListPlacesMessage
    /// Returns 4-element list of places in the room
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum ListPlacesResponse {
        Ok([Option<User>; 4]),
        NotInRoom,
        Unauthenticated,
    }
    impl MessageTrait for ListPlacesResponse {
        const MSG_TYPE: &'static str = "list_places_response";
    }

    /// Answer from server for SelectPlaceMessage
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub enum SelectPlaceResponse {
        Ok,
        NotInRoom,
        PlaceAlreadyTaken,
        Unauthenticated,
    }
    impl MessageTrait for SelectPlaceResponse {
        const MSG_TYPE: &'static str = "select_place_response";
    }

    /// Answer from server for GetCards
    /// Returns list of cards
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum GetCardsResponse {
        Ok { cards: Vec<Card>, position: Player },
        SpectatorNotAllowed,
        NotInRoom,
        Unauthenticated,
    }
    impl MessageTrait for GetCardsResponse {
        const MSG_TYPE: &'static str = "get_cards_response";
    }

    /// Answer from server for TrickMessage
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum MakeBidResponse {
        Ok,
        NotInRoom,
        SpectatorNotAllowed,
        NotYourTurn,
        AuctionNotInProcess,
        InvalidBid,
        Unauthenticated,
    }
    impl MessageTrait for MakeBidResponse {
        const MSG_TYPE: &'static str = "make_bid_response";
    }

    /// Answer from server for TrickMessage
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum MakeTrickResponse {
        Ok,
        NotInRoom,
        SpectatorNotAllowed,
        NotYourTurn,
        TrickNotInProcess,
        InvalidCard,
        Unauthenticated,
    }
    impl MessageTrait for MakeTrickResponse {
        const MSG_TYPE: &'static str = "make_trick_response";
    }

    impl From<&TrickStatus> for MakeTrickResponse {
        fn from(t: &TrickStatus) -> Self {
            match t {
                TrickStatus::Error(TrickError::GameStateMismatch) => {
                    MakeTrickResponse::TrickNotInProcess
                }
                TrickStatus::Error(TrickError::PlayerOutOfTurn) => MakeTrickResponse::NotYourTurn,
                TrickStatus::Error(TrickError::CardNotFound) => MakeTrickResponse::InvalidCard,
                TrickStatus::Error(TrickError::WrongCardSuit) => MakeTrickResponse::InvalidCard, // TODO: different error?
                TrickStatus::TrickInProgress
                | TrickStatus::TrickFinished(_)
                | TrickStatus::DealFinished(_) => MakeTrickResponse::Ok,
            }
        }
    }
}

pub mod server_notification {
    use crate::{Bid, Card, GameResult, GameValue, Player, TrickState};

    use super::*;

    /// Notification sent by server to all users in the room when a new user joins
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct JoinRoomNotification {
        pub user: User,
    }
    impl MessageTrait for JoinRoomNotification {
        const MSG_TYPE: &'static str = "join_room_notification";
    }

    /// Notification sent by server to all users in the room when a user leaves
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct LeaveRoomNotification {
        pub user: User,
    }
    impl MessageTrait for LeaveRoomNotification {
        const MSG_TYPE: &'static str = "leave_room_notification";
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct SelectPlaceNotification {
        pub user: User,
        pub position: Option<Player>,
    }
    impl MessageTrait for SelectPlaceNotification {
        const MSG_TYPE: &'static str = "select_place_notification";
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GameStartedNotification {
        pub start_position: Player,
        pub player_position: [User; 4],
    }
    impl MessageTrait for GameStartedNotification {
        const MSG_TYPE: &'static str = "game_started_notification";
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MakeBidNotification {
        pub player: Player,
        pub bid: Bid,
    }
    impl MessageTrait for MakeBidNotification {
        const MSG_TYPE: &'static str = "make_bid_notification";
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AskBidNotification {
        pub player: Player,
        pub max_bid: Bid,
    }
    impl MessageTrait for AskBidNotification {
        const MSG_TYPE: &'static str = "ask_bid_notification";
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AuctionFinishedNotificationInner {
        pub winner: Player,
        pub max_bid: Bid,
        pub game_value: GameValue,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum AuctionFinishedNotification {
        NoWinner,
        Winner(AuctionFinishedNotificationInner),
    }
    impl MessageTrait for AuctionFinishedNotification {
        const MSG_TYPE: &'static str = "auction_finished_notification";
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AskTrickNotification {
        pub player: Player,
        pub cards: Vec<Card>,
    }
    impl MessageTrait for AskTrickNotification {
        const MSG_TYPE: &'static str = "ask_trick_notification";
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct TrickFinishedNotification {
        pub taker: Player,
        pub cards: Vec<Card>,
    }
    impl MessageTrait for TrickFinishedNotification {
        const MSG_TYPE: &'static str = "trick_finished_notification";
    }

    impl From<TrickState> for TrickFinishedNotification {
        fn from(trick: TrickState) -> Self {
            TrickFinishedNotification {
                taker: trick.taker,
                cards: trick.cards,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GameFinishedNotification {
        pub result: Option<GameResult>,
    }
    impl MessageTrait for GameFinishedNotification {
        const MSG_TYPE: &'static str = "game_finished_notification";
    }

    impl From<GameResult> for GameFinishedNotification {
        fn from(result: GameResult) -> Self {
            GameFinishedNotification {
                result: Some(result),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DummyCardsNotification {
        pub cards: Vec<Card>,
        pub dummy: Player,
    }
    impl MessageTrait for DummyCardsNotification {
        const MSG_TYPE: &'static str = "dummy_cards_notification";
    }

    impl DummyCardsNotification {
        pub fn new(cards: Vec<Card>, dummy: Player) -> Self {
            DummyCardsNotification { cards, dummy }
        }
    }
}
