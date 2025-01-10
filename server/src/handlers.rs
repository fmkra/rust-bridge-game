use std::borrow::Cow;

use common::room::RoomId;
use socketioxide::adapter::Room as SRoom;
use socketioxide::operators::RoomParam;

#[derive(Clone)]
pub struct RoomWrapper(pub RoomId);

impl RoomParam for RoomWrapper {
    type IntoIter = std::iter::Once<SRoom>;
    #[inline(always)]
    fn into_room_iter(self) -> Self::IntoIter {
        std::iter::once(Cow::Owned(self.0.as_str().into()))
    }
}
