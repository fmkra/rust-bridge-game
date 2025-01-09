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

pub fn create_info_notification(msg: String, q: Arc<Mutex<VecDeque<Notification>>>) {
    let not = Notification::new(msg, NotificationType::Info);
    create_notification(not, q);
}

pub fn create_error_notification(msg: String, q: Arc<Mutex<VecDeque<Notification>>>) {
    let not = Notification::new(msg, NotificationType::Error);
    create_notification(not, q);
}

pub fn create_notification(not: Notification, q: Arc<Mutex<VecDeque<Notification>>>) {
    let q_clone = Arc::clone(&q);
    tokio::spawn(async move {
        {
            let mut notifications_queue = q_clone.lock().await;
            notifications_queue.push_back(not);
        }

        sleep(Duration::from_secs(5)).await;

        let mut notifications_queue = q_clone.lock().await;
        notifications_queue.pop_front();
    });
}

pub async fn display_notifications(notifications_arc: Arc<Mutex<VecDeque<Notification>>>) {
    let notifications = {
        let notifications_lock = notifications_arc.lock().await;
        notifications_lock.clone()
    };

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
