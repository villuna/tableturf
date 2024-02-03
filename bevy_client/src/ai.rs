use rand::{seq::{SliceRandom, IteratorRandom}, Rng};
use bevy::prelude::*;
use crate::{board::Board, cards::{CardData, possible_card_placements}, game::Move, Player};

// An AiOpponent implements some algorithm that calculates what the next turn is given the current
// state of the board and other info
// Send + Sync is required for it to be a resource
pub(crate) trait AIOpponent: Send + Sync {
    // Takes in the board and the opponent's hand and returns which card to play and where.
    fn next_move(&mut self, board: &Board, hand: &[CardData; 4], special_points: usize) -> Move;
}

#[derive(Resource)]
pub(crate) struct Opponent {
    ai: Box<dyn AIOpponent>,
    deck: &'static [CardData],
    discard: Vec<usize>,
    hand: [usize; 4],
    special_points: usize,
}

impl Opponent {
    pub(crate) fn new<AI: AIOpponent + 'static>(ai: AI, deck: &'static [CardData]) -> Self {
        let mut rng = rand::thread_rng();
        assert_eq!(deck.len(), 15);
        let hand = (0..15).choose_multiple(&mut rng, 4);
        assert_eq!(hand.len(), 4);
        let hand = [hand[0], hand[1], hand[2], hand[3]];

        Self {
            ai: Box::new(ai),
            deck,
            discard: Vec::new(),
            hand,
            special_points: 0,
        }
    }

    pub(crate) fn make_move(&mut self, board: &Board) -> Move {
        let cards = self.hand.map(|i| self.deck[i]);
        let move_made = self.ai.next_move(board, &cards, self.special_points);
        let mut rng = rand::thread_rng();

        let removed = match move_made {
            Move::Pass(card_id) => {
                self.special_points += 1;
                card_id
            }

            Move::Play { card_id, special, .. } => {
                if special {
                    self.special_points -= self.deck[self.hand[card_id]].special_cost;
                }
                card_id
            }
        };

        self.special_points += 1;
        self.discard.push(self.hand[removed]);

        let drawn = (0..15)
            .filter(|i| !self.discard.contains(i))
            .choose(&mut rng)
            .unwrap();

        self.hand[removed] = drawn;

        move_made
    }

    pub(crate) fn deck(&self) -> &'static [CardData] { self.deck }
}

// An opponent that makes moves at "random".
pub(crate) struct RandomMove;

impl AIOpponent for RandomMove {
    fn next_move(&mut self, board: &Board, hand: &[CardData; 4], special_points: usize) -> Move {
        let mut rng = rand::thread_rng();

        // Players sometimes pass even if they *could* play something else so lets add a tiny
        // probability of that happening with this opponent.
        let pass_probability = 0.1;
        if rng.gen_range(0.0..1.0) < pass_probability {
            return Move::Pass(rng.gen_range(0..4));
        }

        // Pick a random card that has spaces left, then a random space.
        let mut order = hand.iter().enumerate().collect::<Vec<_>>();
        order.shuffle(&mut rng);

        for (index, card) in order {
            let possible = possible_card_placements(board, Player::P2, card, special_points);

            if let Some((pos, rot, special)) = possible.choose(&mut rng) {
                return Move::Play {
                    card_id: index,
                    pos: *pos,
                    rot: *rot,
                    special: *special, 
                };
            }
        }

        Move::Pass(rng.gen_range(0..4))
    }
}
