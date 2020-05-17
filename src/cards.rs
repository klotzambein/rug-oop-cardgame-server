use std::str::FromStr;
use std::{
    fmt::{Debug, Formatter},
    iter::Copied,
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
    Blank,
}

impl Suit {
    pub fn iter() -> Iter<'static, Suit> {
        use Suit::*;
        static SUITS: [Suit; 4] = [Heart, Spade, Club, Diamond];
        SUITS.iter()
    }
}

impl FromStr for Suit {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "h" => Suit::Heart,
            "s" => Suit::Spade,
            "c" => Suit::Club,
            "d" => Suit::Diamond,
            "b" => Suit::Blank,
            _ => Err(())?,
        })
    }
}

impl ToString for Suit {
    fn to_string(&self) -> String {
        match self {
            Suit::Heart => "h".to_owned(),
            Suit::Spade => "s".to_owned(),
            Suit::Club => "c".to_owned(),
            Suit::Diamond => "d".to_owned(),
            Suit::Blank => "b".to_owned(),
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

impl FromStr for Rank {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "k" => Rank::King,
            "q" => Rank::Queen,
            "j" => Rank::Jack,
            "1" => Rank::Ten,
            "9" => Rank::Nine,
            "8" => Rank::Eight,
            "7" => Rank::Seven,
            "6" => Rank::Six,
            "5" => Rank::Five,
            "4" => Rank::Four,
            "3" => Rank::Three,
            "2" => Rank::Two,
            "a" => Rank::Ace,
            _ => Err(())?,
        })
    }
}

impl ToString for Rank {
    fn to_string(&self) -> String {
        match self {
            Rank::King => "k".to_owned(),
            Rank::Queen => "q".to_owned(),
            Rank::Jack => "j".to_owned(),
            Rank::Ten => "1".to_owned(),
            Rank::Nine => "9".to_owned(),
            Rank::Eight => "8".to_owned(),
            Rank::Seven => "7".to_owned(),
            Rank::Six => "6".to_owned(),
            Rank::Five => "5".to_owned(),
            Rank::Four => "4".to_owned(),
            Rank::Three => "3".to_owned(),
            Rank::Two => "2".to_owned(),
            Rank::Ace => "a".to_owned(),
        }
    }
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

impl FromStr for Card {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err(())?
        }
        Ok(Card {
            suit: Suit::from_str(&s[0..1])?,
            rank: Rank::from_str(&s[1..2])?,
        })
    }
}

impl ToString for Card {
    fn to_string(&self) -> String {
        format!("{}{}", self.suit.to_string(), self.rank.to_string())
    }
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
    pub fn new() -> Pile {
        Pile { cards: Vec::new() }
    }
    /// Adds a full normal card-deck without the kings.
    pub fn add_without_kings(mut self) -> Self {
        self.cards.extend(
            Rank::iter()
                .filter(|rank| **rank != Rank::King)
                .flat_map(|rank| Suit::iter().map(move |suit| Card::new(*suit, *rank))),
        );

        self
    }
    /// Adds `n` full suits of blank cards without the kings.
    pub fn add_blank_without_kings(mut self, n: usize) -> Self {
        self.cards.extend(
            Rank::iter()
                .filter(|rank| **rank != Rank::King)
                .flat_map(|rank| std::iter::repeat(Card::new(Suit::Blank, *rank)).take(n)),
        );

        self
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

    pub fn iter(&self) -> Copied<Iter<'_, Card>> {
        self.cards.iter().copied()
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
            cards: Pile::new(),
        }
    }
}
