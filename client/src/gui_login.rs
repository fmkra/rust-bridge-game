use std::sync::Arc;
use serde_json::to_string;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use macroquad::input::KeyCode;

use common::{
    message::client_message::{LoginMessage, LOGIN_MESSAGE},
    user::User,
};

pub fn login_ui(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    nickname_arc: Arc<Mutex<String>>,
) {
    clear_background(Color::from_rgba(50, 115, 85, 255));

    let mut nickname = {
        let nickname_lock = nickname_arc.blocking_lock();
        nickname_lock.clone()
    };

    root_ui().window(hash!(), vec2(10.0, 10.0), vec2(400.0, 150.0), |ui| {
        ui.label(None, "Enter your nickname:");
        ui.input_text(hash!(), "Nickname:", &mut nickname);

        let mut nickname_lock = nickname_arc.blocking_lock();
        *nickname_lock = nickname.clone();

        if ui.button(None, "Confirm") || is_key_pressed(KeyCode::Enter) {
            let login_message = LoginMessage {
                user: User::new(nickname.trim()),
            };

            let socket_clone = socket.clone();
            runtime.spawn(async move {
                socket_clone
                    .emit(LOGIN_MESSAGE, to_string(&login_message).unwrap())
                    .await
                    .unwrap();
            });
        }
    });
}