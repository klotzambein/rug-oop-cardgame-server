use ai::AIPlayer;
use game::{GameState, PlayerAction};

pub mod ai;
pub mod cards;
pub mod co;
pub mod game;

fn main() {
    let mut state = GameState::initial();
    let mut ai0 = AIPlayer::new(0);
    let mut ai1 = AIPlayer::new(1);
    for _ in 0..10 {
        ai0.play_turn(&state)
            .into_iter()
            .for_each(|action| state.perform_player_action(0, action).unwrap());
        ai1.play_turn(&state)
            .into_iter()
            .for_each(|action| state.perform_player_action(1, action).unwrap());
    }
    println!("{:#?}", state);
    co::execute(co::test());
}
