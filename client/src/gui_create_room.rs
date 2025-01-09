use std::sync::Arc;

use common::message::MessageTrait;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use serde_json::to_string;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use common::{
    message::client_message::RegisterRoomMessage,
    room::{RoomId, RoomInfo, Visibility},
};

pub fn create_room_ui(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    room_name_arc: Arc<Mutex<String>>,
) {
    let mut room_name = {
        let state_lock = room_name_arc.blocking_lock();
        state_lock.clone()
    };

    clear_background(Color::from_rgba(50, 115, 85, 255));

    root_ui().window(hash!(), vec2(10.0, 10.0), vec2(400.0, 150.0), |ui| {
        ui.label(None, "Enter Room Name:");
        ui.input_text(hash!(), "Room Name:", &mut room_name);

        // Update the room_name_arc with the current room_name
        {
            let mut state_lock = room_name_arc.blocking_lock();
            *state_lock = room_name.clone();
        }

        if ui.button(None, "Confirm") || is_key_pressed(KeyCode::Enter) {
            let msg = RegisterRoomMessage {
                room_info: RoomInfo {
                    id: RoomId::new(&room_name),
                    visibility: Visibility::Public,
                },
            };

            let socket_clone = socket.clone();
            runtime.spawn(async move {
                socket_clone
                    .emit(RegisterRoomMessage::MSG_TYPE, to_string(&msg).unwrap())
                    .await
                    .unwrap();
            });
        }
    });
}
