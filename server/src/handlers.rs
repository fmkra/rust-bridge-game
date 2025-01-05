use std::borrow::Cow;

use common::message::server_notification::TrickFinishedNotification;
use common::message::MessageTrait;
use common::room::RoomId;
use common::TrickState;
use socketioxide::adapter::Room as SRoom;
use socketioxide::{extract::SocketRef, operators::RoomParam};

#[derive(Clone)]
pub struct RoomWrapper(pub RoomId);

impl RoomParam for RoomWrapper {
    type IntoIter = std::iter::Once<SRoom>;
    #[inline(always)]
    fn into_room_iter(self) -> Self::IntoIter {
        std::iter::once(Cow::Owned(self.0.as_str().into()))
    }
}

pub fn notify_trick_finished(s: &SocketRef, room_id: &RoomId, status: TrickState) {
    s.within(RoomWrapper(room_id.clone()))
        .emit(
            TrickFinishedNotification::MSG_TYPE,
            &TrickFinishedNotification::from(status),
        )
        .unwrap();
}
