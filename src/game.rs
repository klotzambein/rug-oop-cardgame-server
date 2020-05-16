use std::slice::Iter;

use rand::prelude::*;
use rand::rngs::StdRng;

use crate::cards::{Card, Pile, Rank, SpecialPile, Suit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundState {
    player: u8,
    turn_state: TurnState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnState {
    Attack,
    Organize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HousePile {
    One,
    Two,
    Three,
}

impl HousePile {
    pub fn iter() -> Iter<'static, HousePile> {
        use HousePile::*;
        static HOUSE_PILES: [HousePile; 3] = [One, Two, Three];
        HOUSE_PILES.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerPile {
    KingPile,
    HousePile(HousePile),
}

#[derive(Debug, Clone)]
pub enum PlayerAction {
    Attack {
        house_pile: HousePile,
        target_player: Suit,
    },
    AddCardToPile {
        pile: PlayerPile,
        card: Card,
    },
    SwapHousePile(HousePile, HousePile),
    DiscardHand,
}

#[derive(Debug, Clone)]
pub struct GameState {
    rng: StdRng,
    pub round_state: RoundState,
    pub discard_pile: Pile,
    pub stock_pile: Pile,
    pub players: Vec<PlayerState>,
}

impl GameState {
    pub fn initial() -> GameState {
        let mut rng = StdRng::from_entropy();
        let stock_pile = Pile::new_without_kings().shuffled(&mut rng);
        GameState {
            round_state: RoundState {
                player: 0,
                turn_state: TurnState::Attack,
            },
            rng,
            discard_pile: Pile::new_empty(),
            stock_pile,
            players: vec![
                PlayerState::initial(Suit::Heart),
                PlayerState::initial(Suit::Spade),
            ],
        }
    }

    pub fn evaluate_house_pile_value(pile: &SpecialPile) -> u32 {
        match pile.special_card.rank {
            Rank::Jack => pile.cards.count(),
            Rank::Ace => pile.cards.count() + pile.cards.contains_rank(Rank::Two) as u32,
            Rank::Queen => pile.cards.count() * 2,
            _ => panic!("Invalid house pile special card. ({:?})", pile),
        }
    }

    pub fn can_add_to_house_pile(pile: &SpecialPile, card: Card) -> bool {
        (match card.rank {
            Rank::King | Rank::Jack | Rank::Ace | Rank::Queen => false,
            _ => true,
        }) && match pile.special_card.rank {
            Rank::King | Rank::Jack => card.suit == pile.special_card.suit,
            Rank::Ace => {
                let up = pile.cards.contains_rank(card.rank.up());
                let down = pile.cards.contains_rank(card.rank.down());
                !pile.cards.contains_rank(card.rank) && (pile.cards.is_empty() || up || down)
            }
            Rank::Queen => {
                let mut rank_count = [0_u8; 14];
                for card in pile.cards.iter() {
                    rank_count[card.rank as usize] += 1;
                }
                if rank_count.iter().any(|rank| *rank == 1) {
                    rank_count[card.rank as usize] > 0
                } else {
                    true
                }
            }
            _ => panic!("Invalid house pile special card. ({:?})", pile),
        }
    }

    pub fn get_mut_player_by_suit(&mut self, player: Suit) -> Option<&mut PlayerState> {
        self.players.iter_mut().find(|ps| ps.suit == player)
    }

    pub fn perform_player_action(
        &mut self,
        player: u8,
        action: PlayerAction,
    ) -> Result<(), &'static str> {
        if player != self.round_state.player {
            Err("not your turn")?;
        }

        match (self.round_state.turn_state, action) {
            (
                TurnState::Attack,
                PlayerAction::Attack {
                    house_pile,
                    target_player,
                },
            ) => {
                let attack_pile = self.players[player as usize]
                    .get_mut_house_pile(house_pile)
                    .take()
                    .ok_or("chose non existent house pile to attack")?;
                let target_player = self
                    .get_mut_player_by_suit(target_player)
                    .ok_or("attack target player does not exist")?;
                let target_pile = target_player.first_house_pile();
                if let Some(target_pile_ref) = target_pile {
                    let target_pile = target_pile_ref.take().unwrap();
                    let target_value = GameState::evaluate_house_pile_value(&target_pile);
                    let attack_value = GameState::evaluate_house_pile_value(&attack_pile);
                    if attack_value > target_value {
                        self.discard_pile.add(target_pile.special_card);
                        let player = &mut self.players[player as usize];
                        player.hand.add_pile(target_pile.cards);
                        self.discard_pile.add(attack_pile.special_card);
                        self.discard_pile.add_pile(attack_pile.cards);
                    } else {
                        *target_pile_ref = Some(target_pile);
                        target_player.hand.add_pile(attack_pile.cards);
                        self.discard_pile.add(attack_pile.special_card);
                    }
                }
            }
            (TurnState::Organize, PlayerAction::Attack { .. }) => {
                Err("the action is not possible at this point in the turn")?
            }
            (_, PlayerAction::AddCardToPile { pile, card }) => {
                self.round_state.turn_state = TurnState::Organize;
                let player = &mut self.players[player as usize];
                if player.hand.take_card(card) {
                    match card.rank {
                        Rank::King => unreachable!(),
                        Rank::Queen | Rank::Jack | Rank::Ace => match pile {
                            PlayerPile::KingPile => Err("can't put picture cards on king pile")?,
                            PlayerPile::HousePile(p) => {
                                let pile = player.get_mut_house_pile(p);
                                if pile.is_some() {
                                    Err("this pile already exists")?;
                                }
                                *pile = Some(SpecialPile::new(card));
                            }
                        },
                        _ => {
                            let pile = player
                                .get_mut_pile(pile)
                                .ok_or("this pile does not exist")?;
                            if GameState::can_add_to_house_pile(&pile, card) {
                                pile.cards.add(card);
                            } else {
                                Err("this card can not be added to this pile")?;
                            }
                        }
                    }
                }
            }
            (_, PlayerAction::SwapHousePile(a, b)) => {
                self.round_state.turn_state = TurnState::Organize;
                self.players[player as usize].swap_house_piles(a, b);
            }
            (_, PlayerAction::DiscardHand) => {
                let player = &mut self.players[player as usize];
                let hand = std::mem::take(&mut player.hand);
                self.discard_pile.add_pile(hand);
                self.next_player();
            }
        }

        Ok(())
    }

    fn next_player(&mut self) {
        if self.stock_pile.count() < 5 {
            let discard = self.discard_pile.take().shuffled(&mut self.rng);
            self.stock_pile.add_pile(discard);
        }
        let hand = self.stock_pile.take_up_to_n(5);
        self.round_state.turn_state = TurnState::Attack;

        self.round_state.player += 1;
        self.round_state.player %= self.players.len() as u8;

        self.players[self.round_state.player as usize]
            .hand
            .add_pile(hand);
    }
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub suit: Suit,
    pub king_pile: SpecialPile,
    pub house_pile_1: Option<SpecialPile>,
    pub house_pile_2: Option<SpecialPile>,
    pub house_pile_3: Option<SpecialPile>,
    pub hand: Pile,
}

impl PlayerState {
    pub fn initial(suit: Suit) -> PlayerState {
        PlayerState {
            suit,
            king_pile: SpecialPile::new(Card::new(suit, Rank::King)),
            house_pile_1: None,
            house_pile_2: None,
            house_pile_3: None,
            hand: Pile::new_empty(),
        }
    }

    pub fn get_mut_pile(&mut self, pile: PlayerPile) -> Option<&mut SpecialPile> {
        match pile {
            PlayerPile::KingPile => Some(&mut self.king_pile),
            PlayerPile::HousePile(pile) => self.get_mut_house_pile(pile).as_mut(),
        }
    }

    pub fn get_house_pile(&self, pile: HousePile) -> &Option<SpecialPile> {
        match pile {
            HousePile::One => &self.house_pile_1,
            HousePile::Two => &self.house_pile_2,
            HousePile::Three => &self.house_pile_3,
        }
    }

    pub fn get_mut_house_pile(&mut self, pile: HousePile) -> &mut Option<SpecialPile> {
        match pile {
            HousePile::One => &mut self.house_pile_1,
            HousePile::Two => &mut self.house_pile_2,
            HousePile::Three => &mut self.house_pile_3,
        }
    }

    pub fn swap_house_piles(&mut self, a: HousePile, b: HousePile) {
        use std::mem::swap;
        use HousePile::*;
        match (a, b) {
            (One, Two) | (Two, One) => swap(&mut self.house_pile_1, &mut self.house_pile_2),
            (Three, Two) | (Two, Three) => swap(&mut self.house_pile_3, &mut self.house_pile_2),
            (One, Three) | (Three, One) => swap(&mut self.house_pile_1, &mut self.house_pile_3),
            (One, One) | (Two, Two) | (Three, Three) => (),
        }
    }

    pub fn first_house_pile(&mut self) -> Option<&mut Option<SpecialPile>> {
        if self.house_pile_1.is_some() {
            Some(&mut self.house_pile_1)
        } else if self.house_pile_2.is_some() {
            Some(&mut self.house_pile_2)
        } else if self.house_pile_3.is_some() {
            Some(&mut self.house_pile_3)
        } else {
            None
        }
    }

    pub fn house_piles(&self) -> Vec<(HousePile, &SpecialPile)> {
        let mut piles = Vec::with_capacity(3);
        if let Some(p) = &self.house_pile_1 {
            piles.push((HousePile::One, p));
        }
        if let Some(p) = &self.house_pile_2 {
            piles.push((HousePile::Two, p));
        }
        if let Some(p) = &self.house_pile_3 {
            piles.push((HousePile::Three, p));
        }
        piles
    }
}