use common::{user::User, Player};

pub fn update_user_seat(seats: &mut [Option<User>; 4], user: User, new_position: Option<Player>) {
    // Remove user from previously occupied position
    seats.iter_mut().for_each(|opt_user| {
        let is_msg_subject = opt_user.as_ref().map(|u| *u == user);
        if let Some(true) = is_msg_subject {
            *opt_user = None;
        }
    });
    // Place on the new seat
    if let Some(seat) = new_position {
        seats[seat.to_usize()] = Some(user);
    }
}
