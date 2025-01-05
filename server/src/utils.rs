use common::{message::MessageTrait, room::RoomId};
use serde::Serialize;
use socketioxide::extract::SocketRef;

use crate::handlers::RoomWrapper;

pub fn send<M>(socket: &SocketRef, message: &M)
where
    M: MessageTrait + Serialize,
{
    socket.emit(M::MSG_TYPE, message).unwrap();
}

pub fn notify<M>(socket: &SocketRef, room: &RoomId, message: &M)
where
    M: MessageTrait + Serialize,
{
    socket
        .within(RoomWrapper(room.clone()))
        .emit(M::MSG_TYPE, message)
        .unwrap();
}

pub fn notify_others<M>(socket: &SocketRef, room: &RoomId, message: &M)
where
    M: MessageTrait + Serialize,
{
    socket
        .to(RoomWrapper(room.clone()))
        .emit(M::MSG_TYPE, message)
        .unwrap();
}
