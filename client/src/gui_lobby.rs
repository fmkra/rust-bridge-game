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
use tokio::sync::Mutex;

pub async fn list_rooms(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    client: Arc<Mutex<GuiClient>>,
) {
    clear_background(Color::from_rgba(50, 115, 85, 255));

    // let rooms = client.lock().await.rooms.clone();
    let mut client_lock = client.lock().await;

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
            client_lock.state = GuiClientState::CreatingRoom;
            // tokio::spawn(async move {
            //     client.lock().await.state = GuiClientState::CreatingRoom;
            // });
        }

        if ui.button(None, "Exit") {
            std::process::exit(0);
        }

        // Room list appears below the buttons
        for room in &client_lock.rooms.clone() {
            ui.group(hash!(room), vec2(400.0, 50.0), |ui| {
                ui.label(None, &format!("Room ID: {}", room));
                if ui.button(None, "Join") {
                    client_lock.selected_room_name = Some(room.clone());

                    let room_id = RoomId::new(room);
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
