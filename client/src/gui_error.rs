use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use macroquad::prelude::*;

pub fn create_error(msg: String, q: Arc<Mutex<VecDeque<String>>>) {
    let q_clone = Arc::clone(&q);
    tokio::spawn(async move {
        {
            let mut errors_queue = q_clone.lock().await;
            errors_queue.push_back(msg);
        }

        sleep(Duration::from_secs(5)).await;

        let mut errors_queue = q_clone.lock().await;
        errors_queue.pop_front();
    });
}

pub async fn display_errors(errors_arc: Arc<Mutex<VecDeque<String>>>) {
    let errors = {
        let errors_lock = errors_arc.lock().await;
        errors_lock.clone()
    };

    let mut y_offset = screen_height() - 50.0;

    for error in errors.iter() {
        let rect_width = 300.0;
        let rect_height = 30.0;
        let rect_x = screen_width() - rect_width - 20.0;
        let rect_y = y_offset;

        draw_rectangle(rect_x, rect_y, rect_width, rect_height, RED);

        draw_text(
            error,
            rect_x + 10.0,
            rect_y + 20.0,
            16.0,
            WHITE,
        );

        y_offset -= rect_height + 10.0;
    }
}