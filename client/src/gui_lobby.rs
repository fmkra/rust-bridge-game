use crate::gui_client::GuiClientState;

use std::sync::Arc;
use serde_json::to_string;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use common::{
    message::client_message::{ListRoomsMessage, JoinRoomMessage, LIST_ROOMS_MESSAGE, JOIN_ROOM_MESSAGE},
    room::RoomId,
};

pub fn list_rooms(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    rooms_arc: Arc<Mutex<Vec<String>>>,
    state_arc: Arc<Mutex<GuiClientState>>,
) {
    clear_background(Color::from_rgba(50, 115, 85, 255));

    let rooms = {
        let rooms_lock = rooms_arc.blocking_lock(); // Lock rooms briefly to clone
        rooms_lock.clone()
    };

    root_ui().window(hash!(), vec2(10.0, 10.0), vec2(400.0, 400.0), |ui| {
        ui.label(None, "Available Rooms:");

        // Align buttons horizontally by creating two separate groups
        if ui.button(None, "Refresh") {
            let socket_clone = socket.clone();
            runtime.spawn(async move {
                socket_clone
                    .emit(LIST_ROOMS_MESSAGE, to_string(&ListRoomsMessage {}).unwrap())
                    .await
                    .unwrap();
            });
        }

        if ui.button(None, "Create a room") {
            let mut state_lock = state_arc.blocking_lock(); // Lock state briefly to modify
            *state_lock = GuiClientState::CreatingRoom;
        }
        
        // Room list appears below the buttons
        for room in &rooms {
            ui.group(hash!(room), vec2(400.0, 50.0), |ui| {
                ui.label(None, &format!("Room ID: {}", room));
                if ui.button(None, "Join") {
                    let room_id = RoomId::new(room);
                    let socket_clone = socket.clone();
                    runtime.spawn(async move {
                        socket_clone
                            .emit(JOIN_ROOM_MESSAGE, to_string(&JoinRoomMessage { room_id }).unwrap())
                            .await
                            .unwrap();
                    });
                }
            });
        }
    });
}