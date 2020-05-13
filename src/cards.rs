use std::{
    fmt::{Debug, Formatter},
    slice::Iter,
};

use rand::prelude::*;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Heart,
    Spade,
    Club,
    Diamond,
}

impl Suit {
    pub fn iter() -> Iter<'static, Suit> {
        use Suit::*;
        static SUITS: [Suit; 4] = [Heart, Spade, Club, Diamond];
        SUITS.iter()
    }

    pub fn next(self) -> Suit {
        match self {
            Suit::Heart => Suit::Diamond,
            Suit::Spade => Suit::Heart,
            Suit::Club => Suit::Spade,
            Suit::Diamond => Suit::Club,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    King = 13,
    Queen = 12,
    Jack = 11,
    Ten = 10,
    Nine = 9,
    Eight = 8,
    Seven = 7,
    Six = 6,
    Five = 5,
    Four = 4,
    Three = 3,
    Two = 2,
    Ace = 1,
}

impl Rank {
    pub fn iter() -> Iter<'static, Rank> {
        use Rank::*;
        static RANKS: [Rank; 13] = [
            King, Queen, Jack, Ten, Nine, Eight, Seven, Six, Five, Four, Three, Two, Ace,
        ];
        RANKS.iter()
    }

    pub fn down(self) -> Rank {
        match self {
            Rank::King => Rank::Queen,
            Rank::Queen => Rank::Jack,
            Rank::Jack => Rank::Ten,
            Rank::Ten => Rank::Nine,
            Rank::Nine => Rank::Eight,
            Rank::Eight => Rank::Seven,
            Rank::Seven => Rank::Six,
            Rank::Six => Rank::Five,
            Rank::Five => Rank::Four,
            Rank::Four => Rank::Three,
            Rank::Three => Rank::Two,
            Rank::Two => Rank::Ace,
            Rank::Ace => Rank::King,
        }
    }

    pub fn up(self) -> Rank {
        match self {
            Rank::King => Rank::Ace,
            Rank::Queen => Rank::King,
            Rank::Jack => Rank::Queen,
            Rank::Ten => Rank::Jack,
            Rank::Nine => Rank::Ten,
            Rank::Eight => Rank::Nine,
            Rank::Seven => Rank::Eight,
            Rank::Six => Rank::Seven,
            Rank::Five => Rank::Six,
            Rank::Four => Rank::Five,
            Rank::Three => Rank::Four,
            Rank::Two => Rank::Three,
            Rank::Ace => Rank::Two,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Debug for Card {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("{:?} of {:?}s", self.rank, self.suit))?;
        Ok(())
    }
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Card {
        Card { suit, rank }
    }
}

#[derive(Clone, Default)]
pub struct Pile {
    cards: Vec<Card>,
}

impl Pile {
    pub fn new_empty() -> Pile {
        Pile { cards: Vec::new() }
    }
    pub fn new_without_kings() -> Pile {
        let cards = Rank::iter()
            .filter(|rank| **rank != Rank::King)
            .flat_map(|rank| Suit::iter().map(move |suit| Card::new(*suit, *rank)))
            .collect();

        Pile { cards }
    }

    pub fn shuffle(&mut self, rng: &mut impl Rng) {
        self.cards.shuffle(rng);
    }

    pub fn shuffled(mut self, rng: &mut impl Rng) -> Pile {
        self.shuffle(rng);
        self
    }

    pub fn count(&self) -> u32 {
        self.cards.len() as u32
    }

    pub fn contains_rank(&self, rank: Rank) -> bool {
        self.cards.iter().any(|card| card.rank == rank)
    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn add_pile(&mut self, mut pile: Pile) {
        self.cards.append(&mut pile.cards);
    }

    pub fn take(&mut self) -> Pile {
        let cards = std::mem::take(&mut self.cards);
        Pile { cards }
    }

    pub fn take_card(&mut self, card: Card) -> bool {
        let idx = self.cards.iter().position(|c| *c == card);
        if let Some(idx) = idx {
            self.cards.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn take_n(&mut self, n: u32) -> Option<Pile> {
        if n as usize > self.cards.len() {
            None
        } else {
            let cards = self.cards.split_off(self.cards.len() - n as usize);
            Some(Pile { cards })
        }
    }

    pub fn take_up_to_n(&mut self, n: u32) -> Pile {
        let cards = if n as usize >= self.cards.len() {
            std::mem::take(&mut self.cards)
        } else {
            self.cards.split_off(self.cards.len() - n as usize)
        };
        Pile { cards }
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, Card> {
        self.cards.iter()
    }
}

impl Debug for Pile {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.write_str("Pile ")?;
        fmt.debug_list().entries(self.cards.iter().rev()).finish()?;
        Ok(())
    }
}

/// A Pile with at least one card, this card specifies what card can go on the
/// pile and how the pile is interpreted.
#[derive(Debug, Clone)]
pub struct SpecialPile {
    pub special_card: Card,
    pub cards: Pile,
}

impl SpecialPile {
    pub fn new(special_card: Card) -> SpecialPile {
        SpecialPile {
            special_card,
            cards: Pile::new_empty(),
        }
    }
}
