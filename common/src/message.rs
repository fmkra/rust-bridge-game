use serde::{Deserialize, Serialize};

use crate::{
    room::{RoomId, RoomInfo},
    user::User,
};

/// Messages sent from client to server
pub mod client_message {
    use crate::{Bid, Card, Player};

    use super::*;

    pub const LOGIN_MESSAGE: &str = "login";

    /// Message sent by client when attempting to login
    /// Server answers with LoginResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct LoginMessage {
        pub user: User,
    }

    pub const LIST_ROOMS_MESSAGE: &str = "list_rooms";

    /// Message sent by client when requesting list of public rooms
    /// Server answers with ListRoomsResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ListRoomsMessage {}

    pub const REGISTER_ROOM_MESSAGE: &str = "register_room";

    /// Message sent by client when attempting to register a new room
    /// Server answers with RegisterRoomResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RegisterRoomMessage {
        pub room_info: RoomInfo,
    }

    pub const JOIN_ROOM_MESSAGE: &str = "join_room";

    /// Message sent by client when attempting to join a room
    /// Server answers with JoinRoomResponse message
    /// Server sends JoinRoomNotification to all users in the room
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct JoinRoomMessage {
        pub room_id: RoomId,
    }

    pub const LEAVE_ROOM_MESSAGE: &str = "leave_room";

    /// Message sent by client when attempting to leave a room
    /// Server answers with LeaveRoomResponse message
    /// Server sends LeaveRoomNotification to all users in the room
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct LeaveRoomMessage {}

    pub const LIST_PLACES_MESSAGE: &str = "list_places";

    /// Message sent by client when requesting list of places in the room
    /// Server answers with ListPlacesResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ListPlacesMessage {}

    pub const SELECT_PLACE_MESSAGE: &str = "select_place";

    /// Message sent by client when selecting a place in the room
    /// Server answers with UserSelectedPositionMessage message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct SelectPlaceMessage {
        pub position: Option<Player>,
    }

    pub const GET_CARDS_MESSAGE: &str = "get_cards";

    /// Message sent by client when requesting his list of cards
    /// Server answers with GetCardsResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GetCardsMessage {}

    pub const MAKE_BID_MESSAGE: &str = "make_bid";

    /// Message sent by client when making a bid
    /// Server answers with MakeBidResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MakeBidMessage {
        pub bid: Bid,
    }

    pub const MAKE_TRICK_MESSAGE: &str = "make_trick";

    /// Message sent by client when making a trick
    /// Server answers with MakeTrickResponse message
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MakeTrickMessage {
        pub card: Card,
    }
}

pub mod server_response {
    use crate::{Card, Player};

    use super::*;

    pub const LOGIN_RESPONSE: &str = "login_response";

    /// Answer from server for LoginMessage
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum LoginResponse {
        Ok,
        UsernameAlreadyExists,
        UserAlreadyLoggedIn,
    }

    pub const LIST_ROOMS_RESPONSE: &str = "list_rooms_response";

    /// Answer from server for ListRoomsMessage
    /// Returns list of ids of public rooms
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ListRoomsResponse {
        pub rooms: Vec<RoomId>,
    }

    pub const REGISTER_ROOM_RESPONSE: &str = "register_room_response";

    /// Answer from server for RegisterRoomMessage
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub enum RegisterRoomResponse {
        Ok,
        RoomIdAlreadyExists,
        Unauthenticated,
    }

    pub const JOIN_ROOM_RESPONSE: &str = "join_room_response";

    /// Answer from server for JoinRoomMessage
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum JoinRoomResponse {
        Ok,
        AlreadyInRoom,
        RoomNotFound,
        Unauthenticated,
    }

    pub const LEAVE_ROOM_RESPONSE: &str = "leave_room_response";

    /// Answer from server for LeaveRoomMessage
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum LeaveRoomResponse {
        Ok,
        NotInRoom,
        Unauthenticated,
    }

    pub const LIST_PLACES_RESPONSE: &str = "list_places_response";

    /// Answer from server for ListPlacesMessage
    /// Returns 4-element list of places in the room
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum ListPlacesResponse {
        Ok([Option<User>; 4]),
        NotInRoom,
        Unauthenticated,
    }

    pub const SELECT_PLACE_RESPONSE: &str = "select_place_response";

    /// Answer from server for SelectPlaceMessage
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub enum SelectPlaceResponse {
        Ok,
        NotInRoom,
        PlaceAlreadyTaken,
        Unauthenticated,
    }

    pub const GET_CARDS_RESPONSE: &str = "get_cards_response";

    /// Answer from server for GetCards
    /// Returns list of cards
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum GetCardsResponse {
        Ok { cards: Vec<Card>, position: Player },
        SpectatorNotAllowed,
        NotInRoom,
        Unauthenticated,
    }

    pub const MAKE_BID_RESPONSE: &str = "make_bid_response";

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

    pub const MAKE_TRICK_RESPONSE: &str = "make_trick_response";

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
}

pub mod server_notification {
    use crate::{Bid, Card, GameResult, Player, TrickState};

    use super::*;

    pub const JOIN_ROOM_NOTIFICATION: &str = "join_room_notification";

    /// Notification sent by server to all users in the room when a new user joins
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct JoinRoomNotification {
        pub user: User,
    }

    pub const LEAVE_ROOM_NOTIFICATION: &str = "leave_room_notification";

    /// Notification sent by server to all users in the room when a user leaves
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct LeaveRoomNotification {
        pub user: User,
    }

    pub const SELECT_PLACE_NOTIFICATION: &str = "select_place_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct SelectPlaceNotification {
        pub user: User,
        pub position: Option<Player>,
    }

    pub const GAME_STARTED_NOTIFICATION: &str = "game_started_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GameStartedNotification {
        pub start_position: Player,
        pub player_position: [User; 4],
    }

    pub const MAKE_BID_NOTIFICATION: &str = "make_bid_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MakeBidNotification {
        pub player: Player,
        pub bid: Bid,
    }

    pub const ASK_BID_NOTIFICATION: &str = "ask_bid_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AskBidNotification {
        pub player: Player,
        pub max_bid: Bid,
    }

    pub const AUCTION_FINISHED_NOTIFICATION: &str = "auction_finished_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AuctionFinishedNotificationInner {
        pub winner: Player,
        pub max_bid: Bid,
    }

    pub type AuctionFinishedNotification = Option<AuctionFinishedNotificationInner>;

    pub const ASK_TRICK_NOTIFICATION: &str = "ask_trick_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AskTrickNotification {
        pub player: Player,
        pub cards: Vec<Card>,
    }

    pub const TRICK_FINISHED_NOTIFICATION: &str = "trick_finished_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct TrickFinishedNotification {
        pub taker: Player,
        pub cards: Vec<Card>,
    }

    impl From<TrickState> for TrickFinishedNotification {
        fn from(trick: TrickState) -> Self {
            TrickFinishedNotification {
                taker: trick.taker,
                cards: trick.cards,
            }
        }
    }

    pub const GAME_FINISHED_NOTIFICATION: &str = "game_finished_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct GameFinishedNotification {
        pub result: Option<GameResult>,
    }

    impl From<GameResult> for GameFinishedNotification {
        fn from(result: GameResult) -> Self {
            GameFinishedNotification {
                result: Some(result),
            }
        }
    }

    pub const DUMMY_CARDS_NOTIFICATION: &str = "dummy_cards_notification";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DummyCardsNotification {
        pub cards: Vec<Card>,
    }

    impl From<Vec<Card>> for DummyCardsNotification {
        fn from(cards: Vec<Card>) -> Self {
            DummyCardsNotification { cards }
        }
    }
}
