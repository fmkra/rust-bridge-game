use std::{future::Future, pin::Pin};

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
pub fn notify<M>(
    socket: &SocketRef,
    room: &RoomId,
    message: M,
) -> Box<dyn SendableNotification + Send + Sync>
where
    M: MessageTrait + Serialize + SendableNotification + Send + Sync + 'static,
{
    socket
        .within(RoomWrapper(room.clone()))
        .emit(M::MSG_TYPE, &message)
        .unwrap();

    Box::new(message)
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

// WATCH OUT! Ugly shit
pub trait SendableNotification: Send + Sync {
    // How did we get here?
    fn send<'a>(&'a self, socket: &'a SocketRef) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

impl<T: MessageTrait + Serialize + Send + Sync> SendableNotification for T {
    fn send<'a>(&'a self, socket: &'a SocketRef) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        let socket = socket.clone();
        Box::pin(async move {
            // send(&socket, self);
            socket
                .emit_with_ack::<_, String>(Self::MSG_TYPE, self)
                .unwrap()
                .await
                .unwrap();
        })
    }
}
