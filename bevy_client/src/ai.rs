use std::sync::Arc;

use crate::{
    board::Board,
    cards::{possible_card_placements, CardData},
    game::{ActorState, Move},
    Player,
};
use rand::{seq::SliceRandom, Rng};

/// An AiOpponent implements some algorithm that calculates what the next turn is given the current
/// state of the board and other info
/// Send + Sync is required for it to be a resource
pub(crate) trait AIOpponent: Send + Sync {
    /// Considers the board and the state of the opponent and returns which move to play
    fn next_move(&mut self, board: &Board, state: &ActorState) -> Move;
}

pub(crate) struct Opponent {
    ai: Box<dyn AIOpponent>,
    state: ActorState,
}

impl Opponent {
    pub(crate) fn new<AI: AIOpponent + 'static, D: Into<Arc<[CardData]>>>(ai: AI, deck: D) -> Self {
        Self {
            ai: Box::new(ai),
            state: ActorState::new(deck),
        }
    }

    pub(crate) fn make_move(&mut self, board: &Board) -> Move {
        let move_made = self.ai.next_move(board, &self.state);
        self.state.make_move(&move_made);

        move_made
    }

    pub(crate) fn deck(&self) -> &[CardData] {
        self.state.deck.as_ref()
    }
}

/// An opponent that makes moves at "random".
pub(crate) struct RandomMove;

impl AIOpponent for RandomMove {
    fn next_move(&mut self, board: &Board, state: &ActorState) -> Move {
        let mut rng = rand::thread_rng();

        // Players sometimes pass even if they *could* play something else so lets add a tiny
        // probability of that happening with this opponent.
        let pass_probability = 0.1;
        if rng.gen_range(0.0..1.0) < pass_probability {
            return Move::Pass(rng.gen_range(0..4));
        }

        // Pick a random card that has spaces left, then a random space.
        let mut order = state
            .hand
            .iter()
            .cloned()
            .map(|i| &state.deck[i])
            .enumerate()
            .collect::<Vec<_>>();

        order.shuffle(&mut rng);

        for (index, card) in order {
            let possible = possible_card_placements(board, Player::P2, card, state.special_points);

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
