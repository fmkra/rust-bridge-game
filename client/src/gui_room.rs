use common::message::MessageTrait;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use serde_json::to_string;
use std::sync::Arc;

use crate::gui_client::GuiClient;

use common::{
    message::client_message::{LeaveRoomMessage, SelectPlaceMessage},
    user::User,
    Player,
};

pub fn room_ui(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &tokio::runtime::Runtime,
    client: &mut GuiClient,
) {
    clear_background(Color::from_rgba(50, 115, 85, 255));

    root_ui().window(hash!(), vec2(10.0, 10.0), vec2(400.0, 400.0), |ui| {
        ui.label(None, &format!("Room Name: {}", client.selected_room_name));

        ui.separator();

        // Buttons: Exit Room and Spectate
        if ui.button(None, "Exit Room") {
            let socket_clone = socket.clone();
            runtime.spawn(async move {
                socket_clone
                    .emit(
                        LeaveRoomMessage::MSG_TYPE,
                        to_string(&LeaveRoomMessage {}).unwrap(),
                    )
                    .await
                    .unwrap();
            });
        }

        // if ui.button(None, "Spectate") {
        //     let socket_clone = socket.clone();
        //     runtime.spawn(async move {
        //         socket_clone
        //             .emit("SPECTATE", serde_json::json!({}))
        //             .await
        //             .unwrap();
        //     });
        // }

        ui.separator();

        // Seat buttons
        const POSITIONS: [Player; 4] = [Player::North, Player::East, Player::South, Player::West];
        const POSITION_NAMES: [&str; 4] = ["North", "East", "South", "West"];
        for ((&position, &position_name), seat) in POSITIONS
            .iter()
            .zip(POSITION_NAMES.iter())
            .zip(client.seats.iter())
        {
            ui.group(hash!(position_name), vec2(400.0, 50.0), |ui| {
                if let Some(user) = seat {
                    ui.label(None, &format!("{}: {}", position_name, user.get_username()));
                } else {
                    if ui.button(None, format!("Join {}", position_name).as_str()) {
                        client.selected_seat = Some(position);
                        let socket_clone = socket.clone();
                        runtime.spawn(async move {
                            socket_clone
                                .emit(
                                    SelectPlaceMessage::MSG_TYPE,
                                    to_string(&SelectPlaceMessage {
                                        position: Some(position),
                                    })
                                    .unwrap(),
                                )
                                .await
                                .unwrap();
                        });
                    }
                }
            });
        }
    });
}
