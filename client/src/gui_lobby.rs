use crate::gui_client::{GuiClient, GuiClientState};

use common::{
    message::{
        client_message::{JoinRoomMessage, ListRoomsMessage},
        MessageTrait,
    },
    room::RoomId,
};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use serde_json::to_string;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub fn list_rooms(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    client: &mut GuiClient,
) {
    clear_background(Color::from_rgba(50, 115, 85, 255));

    root_ui().window(hash!(), vec2(10.0, 10.0), vec2(400.0, 400.0), |ui| {
        ui.label(None, "Available Rooms:");

        // Align buttons horizontally by creating two separate groups
        if ui.button(None, "Refresh") {
            let socket_clone = socket.clone();
            runtime.spawn(async move {
                socket_clone
                    .emit(
                        ListRoomsMessage::MSG_TYPE,
                        to_string(&ListRoomsMessage {}).unwrap(),
                    )
                    .await
                    .unwrap();
            });
        }

        if ui.button(None, "Create a room") {
            client.state = GuiClientState::CreatingRoom;
        }

        if ui.button(None, "Exit") {
            std::process::exit(0);
        }

        // Room list appears below the buttons
        for room in &client.rooms {
            ui.group(hash!(room), vec2(400.0, 50.0), |ui| {
                ui.label(None, &format!("Room ID: {}", room));
                if ui.button(None, "Join") {
                    let room_id = RoomId::new(room);
                    client.selected_room_name = room.clone();
                    let socket_clone = socket.clone();
                    runtime.spawn(async move {
                        socket_clone
                            .emit(
                                JoinRoomMessage::MSG_TYPE,
                                to_string(&JoinRoomMessage { room_id }).unwrap(),
                            )
                            .await
                            .unwrap();
                    });
                }
            });
        }
    });
}
