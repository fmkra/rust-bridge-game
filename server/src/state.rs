// use futures::stream::{StreamExt, TryStreamExt, };
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use futures::stream::StreamExt;
use socketioxide::extract::SocketRef;
use tokio::{sync::RwLock, time::sleep};

use common::{
    message::server_response::RegisterRoomResponse,
    room::{RoomId, RoomInfo, Visibility},
    user::User,
    Game, Player,
};

use crate::utils::SendableNotification;

pub struct RoomState {
    users: HashSet<User>,

    /// Array of 4 players, where None means that the place is empty.
    /// This array is not cleared when player disconnects, so that no other player can take this place when player disconnects.
    player_positions: [Option<User>; 4],

    /// Notifications that were sent to all users and are important for knowledge of game state.
    /// It is used to inform user that disconnected during game.
    sent_notifications: Vec<Box<dyn SendableNotification + Send + Sync>>,

    pub game: Game,
    pub info: RoomInfo,
}

impl RoomState {
    pub fn new(info: RoomInfo) -> Self {
        Self {
            users: HashSet::new(),
            player_positions: [None, None, None, None],
            game: Game::new(),
            sent_notifications: Vec::new(),
            info,
        }
    }

    fn _remove_player_from_positions(&mut self, user: &User) -> bool {
        let mut removed = false;
        self.player_positions.iter_mut().for_each(|pos| {
            if let Some(pos_user) = pos {
                if pos_user == user {
                    removed = true;
                    *pos = None
                }
            }
        });
        removed
    }

    pub async fn user_join_room(&mut self, user: User) {
        self.users.insert(user);

        // TODO: handle error
    }

    pub fn user_leave_room(&mut self, user: &User) -> bool {
        // TODO: remove from player_positions only if game hasn't started
        self._remove_player_from_positions(user);
        self.users.remove(user)
    }

    pub fn user_select_place(&mut self, user: &User, position: Option<Player>) -> bool {
        // TODO: if game already started, don't allow (return false)
        if let Some(new_position) = position {
            let new_position_usize = new_position.to_usize();
            if self.player_positions[new_position_usize].is_none() {
                self._remove_player_from_positions(user);
                self.player_positions[new_position_usize] = Some(user.clone());
                true
            } else {
                false
            }
        } else {
            self._remove_player_from_positions(user)
        }
    }

    pub fn get_player_positions(&self) -> [Option<User>; 4] {
        self.player_positions.clone()
    }

    pub fn find_player_position(&self, user: &User) -> Option<Player> {
        self.player_positions
            .iter()
            .position(|pos| pos.as_ref() == Some(user))
            .map(|pos| Player::from_usize(pos).unwrap())
    }

    pub fn append_notifications(
        &mut self,
        notifications: Vec<Box<dyn SendableNotification + Send + Sync>>,
    ) {
        self.sent_notifications.extend(notifications);
        // self.sent_notifications.reserve(notifications.len());
        // for notification in notifications {
        //     self.sent_notifications.push(Box::new(notification));
        // }
    }

    pub async fn send_notifications(&self, socket: &SocketRef) {
        println!("Sending {} notifications", self.sent_notifications.len());
        for notification in &self.sent_notifications {
            notification.send(socket).await;
            println!("Notification sent");
            sleep(Duration::from_secs(5)).await;
        }
    }
}

#[derive(Clone)]
pub struct ServerStateInner {
    rooms: HashMap<RoomId, Arc<RwLock<RoomState>>>,
    users: HashSet<User>,
}

pub type ServerState = Arc<RwLock<ServerStateInner>>;

impl ServerStateInner {
    pub fn new() -> Self {
        Self {
            users: HashSet::new(),
            rooms: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) -> bool {
        if self.users.contains(&user) {
            false
        } else {
            self.users.insert(user);
            true
        }
    }

    pub fn remove_user(&mut self, user: &User) {
        self.users.remove(user);
    }

    /// Creates a new room with the given `RoomInfo` and returns Arc to it, which should be used to avoid locking ServerState mutex.
    pub async fn add_room(&mut self, info: RoomInfo) -> RegisterRoomResponse {
        let entry = self.rooms.entry(info.id.clone());
        if let Entry::Vacant(entry) = entry {
            let room = RoomState::new(info);
            let arc = Arc::new(RwLock::new(room));
            entry.insert(arc);
            RegisterRoomResponse::Ok
        } else {
            RegisterRoomResponse::RoomIdAlreadyExists
        }
    }

    pub async fn get_room_list(&self) -> Vec<RoomId> {
        futures::stream::iter(self.rooms.values())
            .filter_map(|room| async {
                let mutex = room.read().await;
                if mutex.info.visibility == Visibility::Public {
                    Some(mutex.info.id.clone())
                } else {
                    None
                }
            })
            .collect()
            .await
    }

    pub async fn get_room(&self, room_id: &RoomId) -> Option<Arc<RwLock<RoomState>>> {
        self.rooms.get(room_id).cloned()
    }
}
