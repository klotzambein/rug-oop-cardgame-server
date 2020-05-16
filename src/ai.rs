use crate::cards::{Card, Rank};
use crate::game::{GameState, HousePile, PlayerAction, PlayerPile};

pub struct AIPlayer {
    player_id: usize,
}

impl AIPlayer {
    pub fn new(player_id: usize) -> AIPlayer {
        AIPlayer { player_id }
    }

    fn evaluate_card(&self, state: &GameState, card: Card) -> Vec<f32> {
        state
            .players
            .iter()
            .map(|player| {
                let suit = player.suit;
                let king_card = if suit == card.suit {
                    player.king_pile.cards.count() as f32 / 2.0 + 2.0
                } else {
                    0.0
                };
                let hp1 = player
                    .house_pile_1
                    .as_ref()
                    .map(|pile| GameState::can_add_to_house_pile(pile, card) as u8 as f32)
                    .unwrap_or(0.0);
                let hp2 = player
                    .house_pile_2
                    .as_ref()
                    .map(|pile| GameState::can_add_to_house_pile(pile, card) as u8 as f32)
                    .unwrap_or(0.0);
                let hp3 = player
                    .house_pile_3
                    .as_ref()
                    .map(|pile| GameState::can_add_to_house_pile(pile, card) as u8 as f32)
                    .unwrap_or(0.0);

                hp1 + hp2 + hp3 + king_card
            })
            .collect()
    }

    fn evaluate_house_pile(&self, state: &GameState, pile: impl Iterator<Item = Card>) -> f32 {
        pile.flat_map(|card| {
            self.evaluate_card(state, card)
                .into_iter()
                .enumerate()
                .map(|(i, s)| if i == self.player_id { -s } else { s })
        })
        .sum()
    }

    fn try_put_down_card(&self, state: &GameState, card: Card) -> Option<PlayerPile> {
        match card.rank {
            Rank::King => unreachable!(),
            Rank::Queen | Rank::Jack | Rank::Ace => HousePile::iter()
                .filter_map(|pile| {
                    if state.players[self.player_id]
                        .get_house_pile(*pile)
                        .is_none()
                    {
                        Some(PlayerPile::HousePile(*pile))
                    } else {
                        None
                    }
                })
                .next(),
            _ => {
                if card.suit == state.players[self.player_id].suit {
                    Some(PlayerPile::KingPile)
                } else {
                    state.players[self.player_id]
                        .house_piles()
                        .iter()
                        .filter_map(|(idx, pile)| {
                            if GameState::can_add_to_house_pile(pile, card) {
                                Some(PlayerPile::HousePile(*idx))
                            } else {
                                None
                            }
                        })
                        .next()
                }
            }
        }
    }

    pub fn play_turn(&mut self, state: &GameState) -> Vec<PlayerAction> {
        let mut state = state.clone();
        let mut actions = Vec::new();

        // Do attacks:
        let attack_piles = state.players[self.player_id]
            .house_piles()
            .into_iter()
            .filter_map(|(idx, pile)| {
                let value = self.evaluate_house_pile(&state, pile.cards.iter());
                let strength = GameState::evaluate_house_pile_value(pile) as f32;
                if value < strength {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for idx in attack_piles {
            if let Some((suit, _)) = state
                .players
                .iter_mut()
                .enumerate()
                .filter_map(|(i, p)| {
                    if i == self.player_id {
                        None
                    } else {
                        let suit = p.suit;
                        p.first_house_pile().map(|sp| (suit, sp))
                    }
                })
                .map(|(s, sp)| {
                    (
                        s,
                        GameState::evaluate_house_pile_value(sp.as_ref().unwrap()),
                    )
                })
                .min_by(|(_, val_a), (_, val_b)| val_a.cmp(val_b))
            {
                let attack = PlayerAction::Attack {
                    house_pile: idx,
                    target_player: suit,
                };
                state
                    .perform_player_action(self.player_id as u8, attack.clone())
                    .unwrap();
                actions.push(attack)
            }
        }

        // Put down cards:
        while let Some((card, pile)) = state.players[self.player_id]
            .hand
            .iter()
            .filter_map(|card| {
                self.try_put_down_card(&state, card)
                    .map(|pile| (card, pile))
            })
            .next()
        {
            let action = PlayerAction::AddCardToPile { pile, card };
            actions.push(action.clone());
            state
                .perform_player_action(self.player_id as u8, action)
                .unwrap();
        }

        //TODO: Reorder cards:

        actions.push(PlayerAction::DiscardHand);

        actions
    }
}
