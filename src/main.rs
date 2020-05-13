use game::{GameState, PlayerAction};

pub mod cards;
pub mod co;
pub mod game;

fn main() {
    let mut state = GameState::initial();
    state
        .perform_player_action(0, PlayerAction::DiscardHand)
        .unwrap();
    state
        .perform_player_action(1, PlayerAction::DiscardHand)
        .unwrap();
    state
        .perform_player_action(0, PlayerAction::DiscardHand)
        .unwrap();
    println!("{:#?}", state);
    co::execute(co::test());
}
