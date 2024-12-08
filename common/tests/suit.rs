use common::*;

#[test]
fn suit_order() {
    let suit_clubs = Suit::Clubs;
    let suit_diamonds = Suit::Diamonds;
    let suit_hearts = Suit::Hearts;
    let suit_spades = Suit::Spades;

    assert!(suit_clubs < suit_diamonds);
    assert!(suit_diamonds > suit_clubs);
    assert!(suit_diamonds < suit_hearts);
    assert!(suit_hearts > suit_diamonds);
    assert!(suit_hearts < suit_spades);
    assert!(suit_spades > suit_hearts);
}
