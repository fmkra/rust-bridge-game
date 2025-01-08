use common::user::User;
use common::{
    message::client_message::{
        MakeBidMessage,
        MAKE_BID_MESSAGE,
    },
    Bid, BidType, Card, Player, Suit
};
use serde_json::to_string;
use macroquad::prelude::*;
use macroquad::texture::{load_texture, DrawTextureParams, Texture2D};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

pub async fn preload_textures() -> HashMap<String, Texture2D> {
    let mut textures = HashMap::new();
    let suit_names = ["C", "D", "H", "S", "NT"];

    for row in 1..=7 {
        for &suit in &suit_names {
            let card_name = format!("{}{}", row, suit);
            let card_path = format!("assets/bids/{}.png", card_name);

            if let Ok(texture) = load_texture(&card_path).await {
                textures.insert(card_name, texture);
            }
        }
    }

    let extra_textures = ["DOUBLE", "PASS", "REDOUBLE"];
    for &extra in &extra_textures {
        let card_path = format!("assets/bids/{}.png", extra);
        if let Ok(texture) = load_texture(&card_path).await {
            textures.insert(extra.to_string(), texture);
        }
    }

    textures
}

pub async fn preload_cards() -> HashMap<String, Texture2D> {
    let mut textures = HashMap::new();
    let suits = ["C", "D", "H", "S"];
    let ranks = [
        "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
    ];

    for &rank in &ranks {
        for &suit in &suits {
            let card_name = format!("{}{}", rank, suit);
            let card_path = format!("assets/cards/{}.png", card_name);

            if let Ok(texture) = load_texture(&card_path).await {
                textures.insert(card_name, texture);
            } else {
                println!("Failed to load texture: {}", card_path);
            }
        }
    }

    textures
}

fn place_bid (
    socket: &Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    bid_arc: &Arc<Mutex<Option<Bid>>>,
    placed_bid: Bid,
) {
    {
        let mut bid_val = bid_arc.blocking_lock();
        *bid_val = Some(placed_bid);
    }
    let socket_clone = socket.clone();
    runtime.spawn(async move {
        socket_clone
            .emit(
                MAKE_BID_MESSAGE,
                to_string(&MakeBidMessage {bid: placed_bid}).unwrap(),
            )
            .await
            .unwrap();
    });
}

pub fn play_ui(
    socket: Arc<rust_socketio::asynchronous::Client>,
    runtime: &Runtime,
    player_seat_arc: Arc<Mutex<Option<Player>>>,
    seats_arc: Arc<Mutex<[Option<User>; 4]>>,
    cards_arc: Arc<Mutex<Option<Vec<Card>>>>,
    bid_arc: Arc<Mutex<Option<Bid>>>,
    trick_arc: Arc<Mutex<Option<Card>>>,
    bid_textures: &HashMap<String, Texture2D>,
    card_textures: &HashMap<String, Texture2D>,
) {
    clear_background(Color::from_rgba(50, 115, 85, 255));

    // Retrieve the player's seat
    let player_position = {
        let player_seat_lock = player_seat_arc.blocking_lock();
        *player_seat_lock
    };

    if player_position.is_none() {
        draw_text("You are currently spectating.", 20.0, 20.0, 20.0, WHITE);
        return;
    }

    let player_position = player_position.unwrap();

    // Retrieve seats data
    let seats = {
        let seats_lock = seats_arc.blocking_lock();
        seats_lock.clone()
    };

    // Retrieve player's cards
    let player_cards = {
        let cards_lock = cards_arc.blocking_lock();
        cards_lock.clone()
    };

    // Dynamic rotation logic to keep the player's seat at the bottom
    let bottom_player = player_position;
    let right_player = player_position.skip(3); // The player to the right
    let top_player = player_position.skip(2); // The player across
    let left_player = player_position.skip(1); // The player to the left

    // Determine usernames for each position
    let bottom_username = seats[bottom_player.to_usize()]
        .as_ref()
        .map(|user| user.get_username())
        .unwrap_or("Empty");
    let right_username = seats[right_player.to_usize()]
        .as_ref()
        .map(|user| user.get_username())
        .unwrap_or("Empty");
    let top_username = seats[top_player.to_usize()]
        .as_ref()
        .map(|user| user.get_username())
        .unwrap_or("Empty");
    let left_username = seats[left_player.to_usize()]
        .as_ref()
        .map(|user| user.get_username())
        .unwrap_or("Empty");

    // Center position of the square
    let square_size = 300.0;
    let square_x = (screen_width() - square_size) / 2.0;
    let square_y = (screen_height() - square_size) / 2.0;

    // Rectangle dimensions for the sides
    let rect_width = square_size * 0.8;
    let rect_height = 50.0;

    // Draw the square
    draw_rectangle_lines(square_x, square_y, square_size, square_size, 5.0, WHITE);

    // Text size for labels
    let text_size = 20.0;

    // Helper function to center text
    let center_text = |text: &str, rect_x: f32, rect_y: f32, rect_width: f32, rect_height: f32| {
        let text_width = measure_text(text, None, text_size as u16, 1.0).width;
        let text_x = rect_x + (rect_width - text_width) / 2.0;
        let text_y = rect_y + (rect_height + text_size) / 2.0 - 5.0;
        draw_text(text, text_x, text_y, text_size, WHITE);
    };

    // Bottom (Your seat)
    draw_rectangle(
        square_x + (square_size - rect_width) / 2.0,
        square_y + square_size,
        rect_width,
        rect_height,
        DARKGRAY,
    );
    center_text(
        &format!("{} (You): {}", bottom_player.to_str(), bottom_username),
        square_x + (square_size - rect_width) / 2.0,
        square_y + square_size,
        rect_width,
        rect_height,
    );

    // Right
    draw_rectangle(
        square_x + square_size,
        square_y + (square_size - rect_width) / 2.0,
        rect_height,
        rect_width,
        DARKGRAY,
    );
    center_text(
        &format!("{}: {}", right_player.to_str(), right_username),
        square_x + square_size,
        square_y + (square_size - rect_width) / 2.0,
        rect_height,
        rect_width,
    );

    // Top
    draw_rectangle(
        square_x + (square_size - rect_width) / 2.0,
        square_y - rect_height,
        rect_width,
        rect_height,
        DARKGRAY,
    );
    center_text(
        &format!("{}: {}", top_player.to_str(), top_username),
        square_x + (square_size - rect_width) / 2.0,
        square_y - rect_height,
        rect_width,
        rect_height,
    );

    // Left
    draw_rectangle(
        square_x - rect_height,
        square_y + (square_size - rect_width) / 2.0,
        rect_height,
        rect_width,
        DARKGRAY,
    );
    center_text(
        &format!("{}: {}", left_player.to_str(), left_username),
        square_x - rect_height,
        square_y + (square_size - rect_width) / 2.0,
        rect_height,
        rect_width,
    );

    let grid_x = screen_width() - 350.0; // Start of grid on the top-right corner
    let grid_y = 50.0; // Starting y position
    let grid_cell_size = 60.0;
    let grid_spacing = 10.0;

    let bid_types = [
        BidType::Trump(Suit::Clubs), 
        BidType::Trump(Suit::Diamonds), 
        BidType::Trump(Suit::Hearts), 
        BidType::Trump(Suit::Spades), 
        BidType::NoTrump
    ];
    let suit_names = ["C", "D", "H", "S", "NT"];
    for row in 0u8..7 { // Rows for numbers 1-7
        for col in 0..5 {
            let bid_name = format!("{}{}", row + 1, suit_names[col]);

            if let Some(texture) = bid_textures.get(&bid_name) {
                draw_texture_ex(
                    texture,
                    grid_x + col as f32 * (grid_cell_size + grid_spacing),
                    grid_y + row as f32 * (grid_cell_size + grid_spacing),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(grid_cell_size, grid_cell_size)),
                        ..Default::default()
                    },
                );

                let click_x = grid_x + col as f32 * (grid_cell_size + grid_spacing);
                let click_y = grid_y + row as f32 * (grid_cell_size + grid_spacing);
                if is_mouse_button_pressed(MouseButton::Left)
                    && mouse_position().0 >= click_x
                    && mouse_position().0 <= click_x + grid_cell_size
                    && mouse_position().1 >= click_y
                    && mouse_position().1 <= click_y + grid_cell_size
                {
                    // Unwrap is valid, as row must be between 0 and 7, and bid_types[col] are of valid types
                    let placed_bid = Bid::new(row, bid_types[col]).unwrap();
                    place_bid(&socket, runtime, &bid_arc, placed_bid);

                    println!("Placed bid: {:?}", placed_bid);
                }
            }
        }
    }

    // Additional row for DOUBLE, PASS, REDOUBLE
    let extra_row_y = grid_y + 7 as f32 * (grid_cell_size + grid_spacing);
    let extra_bids = [Bid::Double, Bid::Pass, Bid::Redouble];
    let extra_textures = ["DOUBLE", "PASS", "REDOUBLE"];

    for i in 0usize..3 {
        let texture_x = grid_x + i as f32 * (grid_cell_size + grid_spacing);

        if let Some(texture) = bid_textures.get(extra_textures[i]) {
            draw_texture_ex(
                texture,
                texture_x,
                extra_row_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(grid_cell_size, grid_cell_size)),
                    ..Default::default()
                },
            );

            let click_x = texture_x;
            let click_y = extra_row_y;
            if is_mouse_button_pressed(MouseButton::Left)
                && mouse_position().0 >= click_x
                && mouse_position().0 <= click_x + grid_cell_size
                && mouse_position().1 >= click_y
                && mouse_position().1 <= click_y + grid_cell_size
            {
                let placed_bid = extra_bids[i];
                place_bid(&socket, runtime, &bid_arc, placed_bid);
                
                println!("Placed bid: {:?}", placed_bid);
            }
        }
    }

    if let Some(mut cards) = player_cards {
        // Sort cards by suit, then by rank
        cards.sort_by(|a, b| a.suit.cmp(&b.suit).then(b.rank.cmp(&a.rank)));

        let pile_y = square_y + square_size + 100.0;
        let card_spacing = 30.0; // Overlapping cards horizontally

        let mut x_offset = square_x;
        let mut clicked_card = None; // Track the topmost clicked card

        // Render cards
        for (i, card) in cards.iter().enumerate() {
            let card_name = format!("{}{}", card.rank.to_str(), card.suit.to_str());
            if let Some(texture) = card_textures.get(&card_name) {
                let card_x = x_offset + i as f32 * card_spacing;

                draw_texture_ex(
                    texture,
                    card_x,
                    pile_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(grid_cell_size * 2.0, grid_cell_size * 2.0)),
                        ..Default::default()
                    },
                );
            }
        }

        // Handle clicks in reverse order
        for (i, card) in cards.iter().enumerate().rev() {
            let card_name = format!("{}{}", card.rank.to_str(), card.suit.to_str());
            if let Some(_) = card_textures.get(&card_name) {
                let card_x = x_offset + i as f32 * card_spacing;

                if clicked_card.is_none()
                    && is_mouse_button_pressed(MouseButton::Left)
                    && mouse_position().0 >= card_x
                    && mouse_position().0 <= card_x + grid_cell_size * 2.0
                    && mouse_position().1 >= pile_y
                    && mouse_position().1 <= pile_y + grid_cell_size * 2.0
                {
                    clicked_card = Some(card.clone());
                    break; // Stop checking once the topmost card is clicked
                }
            }
        }

        // Print the topmost clicked card, if any
        if let Some(card) = clicked_card {
            println!("Clicked on card: {:?}", card);
        }
    }
}
