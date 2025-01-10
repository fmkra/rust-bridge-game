use macroquad::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone, Eq, PartialEq)]
pub enum NotificationType {
    Info,
    Error,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Notification {
    msg: String,
    typ: NotificationType,
}

impl Notification {
    pub fn new(msg: String, typ: NotificationType) -> Notification {
        Notification { msg, typ }
    }
}

#[derive(Clone)]
pub struct Notifier {
    notifications: Arc<Mutex<VecDeque<Notification>>>,
}

impl Notifier {
    pub fn new() -> Self {
        Notifier {
            notifications: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn create_info(&self, msg: String) {
        let notif = Notification::new(msg, NotificationType::Info);
        self.create(notif)
    }

    pub fn create_error(&self, msg: String) {
        let notif = Notification::new(msg, NotificationType::Error);
        self.create(notif)
    }
    pub async fn display(&self) {
        let notifications = self.notifications.lock().await.clone();

        let mut y_offset = screen_height() - 50.0;

        for notification in notifications.iter() {
            let rect_width = 350.0;
            let rect_height = 30.0;
            let rect_x = screen_width() - rect_width - 20.0;
            let rect_y = y_offset;

            let background_color = match notification.typ {
                NotificationType::Info => BLUE,
                NotificationType::Error => RED,
            };

            draw_rectangle(rect_x, rect_y, rect_width, rect_height, background_color);

            draw_text(&notification.msg, rect_x + 10.0, rect_y + 20.0, 16.0, WHITE);

            y_offset -= rect_height + 10.0;
        }
    }

    fn create(&self, notif: Notification) {
        let notifications = self.notifications.clone();
        tokio::spawn(async move {
            {
                let mut notifications_queue = notifications.lock().await;
                notifications_queue.push_back(notif);
            }

            sleep(Duration::from_secs(5)).await;

            let mut notifications_queue = notifications.lock().await;
            notifications_queue.pop_front();
        });
    }
}
