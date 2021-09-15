use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    convert::{TryFrom, TryInto},
};

use crate::state::{CardsHand, State, Turn, VerboseState};
use rand::{seq::SliceRandom, thread_rng};

pub trait Strategy {
    fn get_next_move(state: State) -> Option<State>;
}

pub struct Random;

impl Strategy for Random {
    fn get_next_move(state: State) -> Option<State> {
        let vs: VerboseState = state.into();
        let mut rng = thread_rng();
        vs.following_states()
            .as_slice()
            .choose(&mut rng)
            .map(|vs| vs.try_into().unwrap())
    }
}

pub struct Optimal {
    winning_states: OptimalWinningStates,
}

// Cannot be put inside `impl Optimal`: it is unstable.
type OptimalWinningStates = BTreeMap<Turn, HashSet<State>>;

impl Optimal {
    pub fn new(start_state: State) -> Self {
        let mut reachable_states = HashSet::new();
        let mut winning_states = OptimalWinningStates::new();
        let mut queue = VecDeque::new();

        // Phase 1: find all reachable states.
        queue.push_back(start_state);
        reachable_states.insert(start_state);
        // No need to classify start_state as winning or losing - there is no move from such starting state anyway.
        while let Some(s) = queue.pop_front() {
            let following_states = VerboseState::from(s).following_states();
            for vs in following_states {
                let s = State::try_from(&vs).unwrap();
                if reachable_states.contains(&s) {
                    continue;
                }
                reachable_states.insert(s);
                queue.push_back(s);

                if vs.player_hand == CardsHand::EMPTY {
                    winning_states.entry(Turn::Player).or_default().insert(s);
                } else if vs.opponent_hand == CardsHand::EMPTY {
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
            let fs = vs.following_states();
            winning_cnts.clear();

            for vs2 in &fs {
                let s2 = State::try_from(vs2).unwrap();
                for t in [Turn::Player, Turn::Opponent] {
                    if winning_states[&t].contains(&s2) {
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

        // TODO: remove this printing. ============
        let all = reachable_states.len();
        let winning_p = winning_states[&Turn::Player].len();
        let winning_o = winning_states[&Turn::Opponent].len();
        let draws = all - winning_o - winning_p;
        println!(
            "All: {}, winning_p: {}, winning_o: {}, draws: {}",
            all, winning_p, winning_o, draws,
        );
        // TODO: ==================================

        Self { winning_states }
    }
}

impl Strategy for Optimal {
    fn get_next_move(state: State) -> Option<State> {
        let vs: VerboseState = state.into();
        let _fs = vs.following_states();
        todo!()
    }
}
