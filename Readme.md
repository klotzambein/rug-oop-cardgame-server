# OOP card game
This came was mad for the object oriented programming course at RUG.

## Rules
This game is played with four players each player has one suit. One round
consists of every player acting one after the other. The game is won by
collecting all number cards of the player's suit.

### The Cards
The game is being played with a special card deck. This deck consists of a
standard card deck without kings. And another 12*4 blank cards. There are four
blank cards of every rank except the kings. This gives us a total of 96 cards.
These cards will be shuffled into a stock pile and will be delt to the players.
Once the stock pile is empty the discard pile will be shuffled and becomes the
new stock pile.

### Terminology
- **House Pile**: One of three piles a player can attack and defend with.
- **King Pile**: The pile a player collects all number cards of his suit on.
- **Special Card**: The card at the bottom of a pile determining what cards can
  go on it and how it will be valued.
- **Hand**: These cards will be delt every round and are only visible to one player.

### Player's turn.
- At first the player will be delt five cards. No other player can see these
  cards. Note that the player might already have some cards if they were attacked
  by a different player and won.
- Next the player can choose to attack a different player. They must choose one
  of their piles to attack with and then choose a player. The target player's
  piles will be attacked from the pile closest to the king pile first. To see
  who wins the scores of the to piles have to be compared. The higher score
  wins. If they are equal the defending player wins. The winning player will get
  the number cards of the defeated player's attack pile added to their hand.
- After the player attacked they can choose to put down cards on their four
  piles. Which cards can be put where will be described later.
- The player can also reorder their house piles. This is important, because
  attackers will always attack the pile closest to the king pile first.
- Finally a player can end their turn and discard the rest of their hand.

### Special Piles
- **King Pile**: Has no score, and can only hold number cards of the same suit. The
  player wins if this pile has all number cards.
- **Queen Pile**: The score is the amount of cards without the queen card times
  two. This pile can only hold pairs or more cards of the same rank. Once a pair
  is added another pair can also be added. To be able to add pairs it is also
  allowed to add one singular card of one rank.

  For example a queen pile consists of a Queen of Blanks and a Two of Spades.
  Now we can only add another two. After we added one or more twos we can add
  any number card again.
- **Jack Pile**: The score is the amount of cards without the jack card. This
  pile can hold any number card of the same suit as the jack.
- **Ace Pile**: If the ace pile contains a two the score is the total amount of
  cards (including ace). If there is no two the score is the amount of cards
  excluding the ace. This pile must always make a valid straight (excluding the
  ace). However, straights of length one are allowed.

## Joining games
A game can be created for one to four players. After that a game id will be
displayed. This id can be used by other players to join the game. For the games
without four players very primitive AI players will be added to play the game.
The game will start once all players have joined the game.
