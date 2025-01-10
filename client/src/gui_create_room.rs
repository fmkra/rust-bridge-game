use std::sync::Arc;

use common::message::MessageTrait;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use serde_json::to_string;
use tokio::runtime::Runtime;

use common::{
    message::client_message::RegisterRoomMessage,
    room::{RoomId, RoomInfo, Visibility},
};

use crate::gui_client::GuiClient;

pub fn create_room_ui(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    client: &mut GuiClient,
) {
    clear_background(Color::from_rgba(50, 115, 85, 255));

    root_ui().window(hash!(), vec2(10.0, 10.0), vec2(400.0, 150.0), |ui| {
        ui.label(None, "Enter Room Name:");
        ui.input_text(hash!(), "Room Name:", &mut client.selected_room_name);

        if ui.button(None, "Confirm") || is_key_pressed(KeyCode::Enter) {
            let msg = RegisterRoomMessage {
                room_info: RoomInfo {
                    id: RoomId::new(&client.selected_room_name),
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
