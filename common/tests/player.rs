use common::*;

#[test]
fn player_from_u8() {
    let player0 = Player::from_u8(0);
    let player1 = Player::from_u8(1);
    let player2 = Player::from_u8(2);
    let player3 = Player::from_u8(3);
    let player_none = Player::from_u8(4);

    assert_eq!(player0, Some(Player::North));
    assert_eq!(player1, Some(Player::East));
    assert_eq!(player2, Some(Player::South));
    assert_eq!(player3, Some(Player::West));
    assert_eq!(player_none, None);
}

#[test]
fn player_to_u8() {
    let player0 = Player::North;
    let player1 = Player::East;
    let player2 = Player::South;
    let player3 = Player::West;

    assert_eq!(player0.to_u8(), 0);
    assert_eq!(player1.to_u8(), 1);
    assert_eq!(player2.to_u8(), 2);
    assert_eq!(player3.to_u8(), 3);
}

#[test]
fn player_next() {
    let player0 = Player::North;
    let player1 = Player::East;
    let player2 = Player::South;
    let player3 = Player::West;

    assert_eq!(player0.next(), player1);
    assert_eq!(player1.next(), player2);
    assert_eq!(player2.next(), player3);
    assert_eq!(player3.next(), player0);
}