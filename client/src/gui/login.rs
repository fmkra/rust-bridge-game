use common::message::MessageTrait;
use macroquad::input::KeyCode;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use serde_json::to_string;
use std::sync::Arc;
use tokio::runtime::Runtime;

use common::{message::client_message::LoginMessage, user::User};

pub fn login_ui(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    nickname: &mut String,
) {
    clear_background(Color::from_rgba(50, 115, 85, 255));

    root_ui().window(hash!(), vec2(10.0, 10.0), vec2(400.0, 150.0), |ui| {
        ui.label(None, "Enter your nickname:");
        ui.input_text(hash!(), "Nickname:", nickname);

        if ui.button(None, "Confirm") || is_key_pressed(KeyCode::Enter) {
            let login_message = LoginMessage {
                user: User::new(nickname.trim()),
            };

            let socket_clone = socket.clone();
            runtime.spawn(async move {
                socket_clone
                    .emit(LoginMessage::MSG_TYPE, to_string(&login_message).unwrap())
                    .await
                    .unwrap();
            });
        }
    });
}
