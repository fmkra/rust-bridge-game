use common::{message::MessageTrait, room::RoomId};
use serde::Serialize;
use socketioxide::extract::SocketRef;

use crate::{handlers::RoomWrapper, ClientData};

/// Sends given message to user that makes request (given by socket)
pub fn send<M>(socket: &SocketRef, message: &M)
where
    M: MessageTrait + Serialize,
{
    socket.emit(M::MSG_TYPE, message).unwrap();
}

/// Send message to room with given `RoomId``
pub fn notify<M>(socket: &SocketRef, room: &RoomId, message: &M)
where
    M: MessageTrait + Serialize,
{
    socket
        .within(RoomWrapper(room.clone()))
        .emit(M::MSG_TYPE, message)
        .unwrap();
}

/// Send message to everyone in room with given `RoomId` except for use that makes request
pub fn notify_others<M>(socket: &SocketRef, room: &RoomId, message: &M)
where
    M: MessageTrait + Serialize,
{
    socket
        .to(RoomWrapper(room.clone()))
        .emit(M::MSG_TYPE, message)
        .unwrap();
}

/// If user that makes request isn't logged in,
/// it sends given error message and returns None.
/// Otherwise returns Some with `ClientData` of logged user.
pub fn get_client_or_response<M>(socket: &SocketRef, response: &M) -> Option<ClientData>
where
    M: MessageTrait + Serialize,
{
    let data = socket.extensions.get::<ClientData>();
    if data.is_none() {
        send(&socket, response);
    }
    data
}
