use crate::state::{CardsHand, Move, State, Turn, VerboseState};
use rand::{seq::SliceRandom, thread_rng};
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    convert::{TryFrom, TryInto},
};

pub trait Strategy {
    fn get_next_move(&self, state: &VerboseState) -> Option<Move>;
}

pub struct Random;

impl Strategy for Random {
    fn get_next_move(&self, state: &VerboseState) -> Option<Move> {
        let mut rng = thread_rng();
        state.possible_moves().as_slice().choose(&mut rng).cloned()
    }
}

pub struct Optimal {
    winning_states: OptimalWinningStates,
}

// Cannot be put inside `impl Optimal`: it is unstable.
type OptimalWinningStates = BTreeMap<Turn, HashSet<State>>;

impl Optimal {
    // Consider: paralelize construction? Or maybe keep some cache of states?
    pub fn new(start_state: &VerboseState) -> Self {
        let mut reachable_states = HashSet::new();
        let mut winning_states = OptimalWinningStates::new();
        let mut queue = VecDeque::new();

        // Phase 1: find all reachable states.
        let start_state = start_state.try_into().expect("Valid start_state"); // expect: Not ideal, but should be good enough.
        queue.push_back(start_state);
        reachable_states.insert(start_state);
        // No need to classify start_state as winning or losing - there is no move from such starting state anyway.
        while let Some(s) = queue.pop_front() {
            let following_states = VerboseState::from(s).possible_moves();
            for mov in following_states {
                let s = State::try_from(&mov.state).unwrap();
                if reachable_states.contains(&s) {
                    continue;
                }
                reachable_states.insert(s);
                queue.push_back(s);

                if mov.state.player_hand == CardsHand::EMPTY {
                    winning_states.entry(Turn::Player).or_default().insert(s);
                } else if mov.state.opponent_hand == CardsHand::EMPTY {
                    winning_states.entry(Turn::Opponent).or_default().insert(s);
                }
            }
        }

        // Phase 2: propagate down winning states.
        let add_preceding_states = |queue: &mut VecDeque<_>, s: &State| {
            for vs in VerboseState::from(*s).preceding_states() {
                let s = State::try_from(vs).unwrap();
                if reachable_states.contains(&s) {
                    queue.push_back(s);
                }
            }
        };
        for t in [Turn::Player, Turn::Opponent] {
            for s in &winning_states[&t] {
                add_preceding_states(&mut queue, s);
            }
        }

        let mut winning_cnts = BTreeMap::<_, usize>::new();
        while let Some(s) = queue.pop_front() {
            if winning_states[&Turn::Player].contains(&s)
                || winning_states[&Turn::Opponent].contains(&s)
            {
                continue;
            }

            let vs = VerboseState::from(s);
            let fs = vs.possible_moves();
            winning_cnts.clear();

            for mov in &fs {
                let next_s = State::try_from(&mov.state).unwrap();
                for t in [Turn::Player, Turn::Opponent] {
                    if winning_states[&t].contains(&next_s) {
                        *winning_cnts.entry(t).or_default() += 1;
                    }
                }
            }

            if winning_cnts.get(&vs.turn).copied().unwrap_or_default() > 0 {
                winning_states.entry(vs.turn).or_default().insert(s);
                add_preceding_states(&mut queue, &s);
            } else if winning_cnts
                .get(&vs.turn.next())
                .copied()
                .unwrap_or_default()
                == fs.len()
            {
                winning_states.entry(vs.turn.next()).or_default().insert(s);
                add_preceding_states(&mut queue, &s);
            }
        }

        Self { winning_states }
    }

    pub fn get_winning_turn(&self, vs: &VerboseState) -> Option<Turn> {
        let s = vs.try_into().unwrap();
        for t in [Turn::Player, Turn::Opponent] {
            if self
                .winning_states
                .get(&t)
                .map_or(false, |states| states.contains(&s))
            {
                return Some(t);
            }
        }

        None
    }
}

impl Strategy for Optimal {
    fn get_next_move(&self, state: &VerboseState) -> Option<Move> {
        let (mut win, mut draw, mut lose) = (vec![], vec![], vec![]);
        let moves = state.possible_moves();
        for m in moves {
            let s = State::try_from(&m.state).unwrap();
            if self.winning_states[&state.turn].contains(&s) {
                win.push(m);
            } else if self.winning_states[&state.turn.next()].contains(&s) {
                lose.push(m);
            } else {
                draw.push(m);
            }
        }

        let mut rng = thread_rng();
        if let m @ Some(_) = win.choose(&mut rng) {
            m.cloned()
        } else if let m @ Some(_) = draw.choose(&mut rng) {
            m.cloned()
        } else {
            lose.choose(&mut rng).cloned()
        }
    }
}
