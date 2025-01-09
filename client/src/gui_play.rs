use common::message::MessageTrait;
use common::user::User;
use common::{
    message::client_message::{MakeBidMessage, MakeTrickMessage},
    Bid, BidType, Card, Player, Suit,
};
use macroquad::prelude::*;
use macroquad::texture::{load_texture, DrawTextureParams, Texture2D};
use serde_json::to_string;
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

fn place_bid(
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
                MakeBidMessage::MSG_TYPE,
                to_string(&MakeBidMessage { bid: placed_bid }).unwrap(),
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
    current_player_arc: Arc<Mutex<Option<Player>>>,
    dummy_cards_arc: Arc<Mutex<Option<Vec<Card>>>>,
    dummy_player_arc: Arc<Mutex<Option<Player>>>,
    current_placed_cards_arc: Arc<Mutex<[Option<Card>; 4]>>,
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
        return;
    }

    let player_position = player_position.unwrap();

    // Retrieve seats data
    let seats = {
        let seats_lock = seats_arc.blocking_lock();
        seats_lock.clone()
    };

    // Retrieve current player
    let current_player = {
        let current_player_lock = current_player_arc.blocking_lock();
        *current_player_lock
    };

    let dummy_player = {
        let dummy_player_lock = dummy_player_arc.blocking_lock();
        *dummy_player_lock
    };

    let dummy_cards = {
        let dummy_cards_lock = dummy_cards_arc.blocking_lock();
        dummy_cards_lock.clone()
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
    let square_x = 0.3 * screen_width() - square_size / 2.0;
    let square_y = 0.5 * screen_height() - square_size / 2.0;

    // Rectangle dimensions for the sides
    let rect_width = square_size * 0.8;
    let rect_height = 50.0;

    // Draw the square
    draw_rectangle_lines(square_x, square_y, square_size, square_size, 5.0, WHITE);

    // Text size for labels
    let text_size = 30.0;

    // Helper function to center text
    let center_text =
        |text: &str, rect_x: f32, rect_y: f32, rect_width: f32, rect_height: f32, color: Color| {
            let text_width = measure_text(text, None, text_size as u16, 1.0).width;
            let text_x = rect_x + (rect_width - text_width) / 2.0;
            let text_y = rect_y + (rect_height + text_size) / 2.0 - 5.0;
            draw_text(text, text_x, text_y, text_size, color);
        };

    // Determine text color based on the current player
    let get_text_color = |player: Player| {
        if Some(player) == current_player {
            BLUE
        } else {
            WHITE
        }
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
        get_text_color(bottom_player),
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
        get_text_color(right_player),
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
        get_text_color(top_player),
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
        get_text_color(left_player),
    );

    let grid_x = screen_width() - 350.0; // Start of grid on the top-right corner
    let grid_y = 50.0; // Starting y position
    let grid_cell_size = 60.0;
    let grid_spacing = 10.0;

    if let (Some(dummy_cards), Some(dummy_player)) = (dummy_cards, dummy_player) {
        if dummy_player != player_position {
            // Sort dummy cards by suit, then by rank
            let mut dummy_cards_sorted = dummy_cards.clone();
            dummy_cards_sorted.sort_by(|a, b| a.suit.cmp(&b.suit).then(b.rank.cmp(&a.rank)));

            let dummy_card_width = grid_cell_size * 2.0; // Size of each dummy card
            let dummy_card_spacing = 30.0; // Overlapping spacing for dummy cards
            let extra_offset = 100.0; // Increased additional spacing from the table

            // Calculate total width/height of the pile
            let total_pile_length =
                (dummy_cards_sorted.len() as f32 - 1.0) * dummy_card_spacing + dummy_card_width;

            match dummy_player {
                p if p == left_player => {
                    // Center vertically on the left side with additional left offset
                    let pile_y = square_y + (square_size - total_pile_length) / 2.0;
                    let pile_x = square_x - dummy_card_width - 20.0 - extra_offset; // Move further left

                    for (i, card) in dummy_cards_sorted.iter().enumerate() {
                        let card_name = format!("{}{}", card.rank.to_str(), card.suit.to_str());
                        if let Some(texture) = card_textures.get(&card_name) {
                            let card_y = pile_y + i as f32 * dummy_card_spacing;

                            draw_texture_ex(
                                texture,
                                pile_x,
                                card_y,
                                WHITE,
                                DrawTextureParams {
                                    dest_size: Some(Vec2::new(dummy_card_width, dummy_card_width)),
                                    rotation: std::f32::consts::FRAC_PI_2, // Rotate 90 degrees
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
                p if p == right_player => {
                    // Center vertically on the right side with additional right offset
                    let pile_y = square_y + (square_size - total_pile_length) / 2.0;
                    let pile_x = square_x + square_size + 20.0 + extra_offset; // Move further right

                    for (i, card) in dummy_cards_sorted.iter().enumerate() {
                        let card_name = format!("{}{}", card.rank.to_str(), card.suit.to_str());
                        if let Some(texture) = card_textures.get(&card_name) {
                            let card_y = pile_y + i as f32 * dummy_card_spacing;

                            draw_texture_ex(
                                texture,
                                pile_x,
                                card_y,
                                WHITE,
                                DrawTextureParams {
                                    dest_size: Some(Vec2::new(dummy_card_width, dummy_card_width)),
                                    rotation: -std::f32::consts::FRAC_PI_2, // Rotate -90 degrees
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
                p if p == top_player => {
                    // Center horizontally at the top with additional top offset
                    let pile_x = square_x + (square_size - total_pile_length) / 2.0;
                    let pile_y = square_y - dummy_card_width - 20.0 - extra_offset; // Move further up

                    for (i, card) in dummy_cards_sorted.iter().enumerate() {
                        let card_name = format!("{}{}", card.rank.to_str(), card.suit.to_str());
                        if let Some(texture) = card_textures.get(&card_name) {
                            let card_x = pile_x + i as f32 * dummy_card_spacing;

                            draw_texture_ex(
                                texture,
                                card_x,
                                pile_y,
                                WHITE,
                                DrawTextureParams {
                                    dest_size: Some(Vec2::new(dummy_card_width, dummy_card_width)),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Placeholder for Bid Texture
    let bid_texture_name = "1NT";
    let bid_texture_width = grid_cell_size; // Explicitly setting width for consistency
    let bid_texture_height = grid_cell_size; // Explicitly setting height for consistency

    // TODO: DISPLAY BIDS PLACED BY PLAYERS
    // Texture_top: Positioned at top: 0px, left: 120px
    if let Some(bid_texture) = bid_textures.get(bid_texture_name) {
        draw_texture_ex(
            bid_texture,
            square_x + 120.0,
            square_y + 0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(bid_texture_width, bid_texture_height)),
                ..Default::default()
            },
        );

        // Texture_right: Positioned at top: 120px, left: 240px
        draw_texture_ex(
            bid_texture,
            square_x + 240.0,
            square_y + 120.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(bid_texture_width, bid_texture_height)),
                ..Default::default()
            },
        );

        // Texture_bottom: Positioned at top: 240px, left: 120px
        draw_texture_ex(
            bid_texture,
            square_x + 120.0,
            square_y + 240.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(bid_texture_width, bid_texture_height)),
                ..Default::default()
            },
        );

        // Texture_left: Positioned at top: 120px, left: 0px
        draw_texture_ex(
            bid_texture,
            square_x + 0.0,
            square_y + 120.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(bid_texture_width, bid_texture_height)),
                ..Default::default()
            },
        );
    }

    // DISPLAY CARDS TRICKED -----------------------------------------------------------------------
    // Calculate the offset for shifting positions (player_position as bottom = 2)
    let shift_offset = match player_position {
        Player::North => 2, // Shift North to bottom
        Player::East => 1,  // Shift East to bottom
        Player::South => 0, // South is already at the bottom
        Player::West => 3,  // Shift West to bottom
    };

    // Adjust the positions for the placeholders based on rotation
    let placeholder_positions = [
        (square_x + 90.0, square_y + 0.0),   // North
        (square_x + 180.0, square_y + 90.0), // East
        (square_x + 90.0, square_y + 180.0), // South
        (square_x + 0.0, square_y + 90.0),   // West
    ];
    let adjusted_positions: Vec<(f32, f32)> = placeholder_positions
        .iter()
        .cycle()
        .skip(shift_offset)
        .take(4)
        .cloned()
        .collect();

    // Retrieve the current trick cards
    let current_trick = {
        let trick_lock = current_placed_cards_arc.blocking_lock();
        *trick_lock
    };

    // Card dimensions
    let card_texture_width = grid_cell_size * 2.0;
    let card_texture_height = grid_cell_size * 2.0;

    for (i, position) in adjusted_positions.iter().enumerate() {
        let (placeholder_x, placeholder_y) = position;

        // If there's a card in the current trick for this position, display it
        if let Some(card) = current_trick[i] {
            let card_name = format!("{}{}", card.rank.to_str(), card.suit.to_str());
            if let Some(texture) = card_textures.get(&card_name) {
                draw_texture_ex(
                    texture,
                    *placeholder_x,
                    *placeholder_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(card_texture_width, card_texture_height)),
                        ..Default::default()
                    },
                );
            }
        }
        // If there's no card, skip drawing anything, leaving it transparent
    }

    // DISPLAY BIDS TO PLAY --------------------------------------------------------------------------------------
    let bid_types = [
        BidType::Trump(Suit::Clubs),
        BidType::Trump(Suit::Diamonds),
        BidType::Trump(Suit::Hearts),
        BidType::Trump(Suit::Spades),
        BidType::NoTrump,
    ];
    let suit_names = ["C", "D", "H", "S", "NT"];
    for row in 0u8..7 {
        // Rows for numbers 1-7
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
                    // Unwrap is valid, as row must be between 1 and 7, and bid_types[col] are of valid types
                    let placed_bid = Bid::new(row + 1, bid_types[col]).unwrap();
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

    // DISPLAY PLAYER CARDS --------------------------------------------------------------------------------------------------
    if let Some(mut cards) = player_cards {
        // Sort cards by suit, then by rank
        cards.sort_by(|a, b| a.suit.cmp(&b.suit).then(b.rank.cmp(&a.rank)));

        let pile_y = square_y + square_size + 100.0;
        let card_spacing = 30.0; // Overlapping cards horizontally
        let card_width = grid_cell_size * 2.0; // Each card's width

        // Calculate the total width of the pile and center it
        let total_pile_width = (cards.len() as f32 - 1.0) * card_spacing + card_width;
        let x_offset = square_x + (square_size - total_pile_width) / 2.0;

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
                        dest_size: Some(Vec2::new(card_width, card_width)),
                        ..Default::default()
                    },
                );
            }
        }

        // Handle clicks in reverse order to respect the overlapping priority
        for (i, card) in cards.iter().enumerate().rev() {
            let card_name = format!("{}{}", card.rank.to_str(), card.suit.to_str());
            if let Some(_) = card_textures.get(&card_name) {
                let card_x = x_offset + i as f32 * card_spacing;

                if clicked_card.is_none()
                    && is_mouse_button_pressed(MouseButton::Left)
                    && mouse_position().0 >= card_x
                    && mouse_position().0 <= card_x + card_width
                    && mouse_position().1 >= pile_y
                    && mouse_position().1 <= pile_y + card_width
                {
                    clicked_card = Some(card.clone());
                    break; // Stop checking once the topmost card is clicked
                }
            }
        }

        // Handle the clicked card
        if let Some(card) = clicked_card {
            println!("CLICKED: {:?}", card);
            let socket_clone = socket.clone();
            let mut trick_val = trick_arc.blocking_lock();
            *trick_val = Some(card);
            runtime.spawn(async move {
                socket_clone
                    .emit(
                        MakeBidMessage::MSG_TYPE,
                        to_string(&MakeTrickMessage { card }).unwrap(),
                    )
                    .await
                    .unwrap();
            });
        }
    }
}
