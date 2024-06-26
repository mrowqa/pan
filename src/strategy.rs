use crate::state::{CardsHand, Move, State, Turn, VerboseState};
use rand::{seq::SliceRandom, thread_rng};
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    convert::{TryFrom, TryInto},
    fs,
    path::Path,
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

pub struct Optimal<'a> {
    cache: &'a OptimalCache,
}

// Cannot be put inside `impl Optimal`: it is unstable.
type OptimalWinningStates = BTreeMap<Option<Turn>, HashSet<State>>;

impl<'a> Optimal<'a> {
    // Consider: paralelize construction? Or maybe keep some cache of states?
    pub fn new_with_mut_cache(start_state: &VerboseState, cache: &'a mut OptimalCache) -> Self {
        let mut new_reachable_states = HashSet::new();
        let mut queue = VecDeque::new();
        let mut winning_queue = VecDeque::new();

        // Phase 1: find all reachable states (unknown to already built cache).
        let start_state = start_state.try_into().expect("Valid start_state"); // expect: Not ideal, but should be good enough.
        queue.push_back(start_state);
        if cache.get_state_winningness(start_state).is_none() {
            new_reachable_states.insert(start_state);
        }
        // No need to classify start_state as winning or losing - there is no move from such starting state anyway.
        while let Some(s) = queue.pop_front() {
            let following_states = VerboseState::from(s).possible_moves();
            for mov in following_states {
                let s = State::try_from(&mov.state).unwrap();
                if new_reachable_states.contains(&s) || cache.get_state_winningness(s).is_some() {
                    continue;
                }
                new_reachable_states.insert(s);
                queue.push_back(s);

                if mov.state.player_hand == CardsHand::EMPTY {
                    cache
                        .states
                        .entry(Some(Turn::Player))
                        .or_default()
                        .insert(s);
                    winning_queue.push_back(s);
                } else if mov.state.opponent_hand == CardsHand::EMPTY {
                    cache
                        .states
                        .entry(Some(Turn::Opponent))
                        .or_default()
                        .insert(s);
                    winning_queue.push_back(s);
                }
            }
        }

        // Phase 2: propagate down winning states.
        let add_preceding_states = |queue: &mut VecDeque<_>, s: State| {
            for vs in VerboseState::from(s).preceding_states() {
                let s = State::try_from(vs).unwrap();
                if new_reachable_states.contains(&s) {
                    queue.push_back(s);
                }
            }
        };
        while let Some(s) = winning_queue.pop_front() {
            add_preceding_states(&mut queue, s);
        }

        let mut winning_cnts = BTreeMap::<_, usize>::new();
        while let Some(s) = queue.pop_front() {
            if cache.states[&Some(Turn::Player)].contains(&s)
                || cache.states[&Some(Turn::Opponent)].contains(&s)
            {
                continue;
            }

            let vs = VerboseState::from(s);
            let pm = vs.possible_moves();
            winning_cnts.clear();

            for mov in &pm {
                let next_s = State::try_from(&mov.state).unwrap();
                for t in [Turn::Player, Turn::Opponent] {
                    if cache.states[&Some(t)].contains(&next_s) {
                        *winning_cnts.entry(t).or_default() += 1;
                    }
                }
            }

            if winning_cnts.get(&vs.turn).copied().unwrap_or_default() > 0 {
                cache.states.entry(Some(vs.turn)).or_default().insert(s);
                add_preceding_states(&mut queue, s);
            } else if winning_cnts
                .get(&vs.turn.next())
                .copied()
                .unwrap_or_default()
                == pm.len()
            {
                cache
                    .states
                    .entry(Some(vs.turn.next()))
                    .or_default()
                    .insert(s);
                add_preceding_states(&mut queue, s);
            }
        }

        // Add all remaining states as draw ones.
        for s in &new_reachable_states {
            if cache.get_state_winningness(*s).is_none() {
                cache.states.entry(None).or_default().insert(*s);
            }
        }

        Self { cache }
    }

    pub fn get_winning_turn(&self, vs: &VerboseState) -> Option<Turn> {
        let s = vs.try_into().unwrap();
        self.cache
            .get_state_winningness(s)
            .expect("Properly initialized strategy.")
    }
}

impl<'a> Strategy for Optimal<'a> {
    fn get_next_move(&self, state: &VerboseState) -> Option<Move> {
        let (mut win, mut draw, mut lose) = (vec![], vec![], vec![]);
        let moves = state.possible_moves();
        for m in moves {
            match self.get_winning_turn(&m.state) {
                None => draw.push(m),
                Some(t) if t == state.turn => win.push(m),
                _ => lose.push(m),
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

pub struct OptimalCache {
    states: OptimalWinningStates,
}

const OPTIMAL_SERIALIZATION_ORDER: [Option<Turn>; 3] =
    [Some(Turn::Player), Some(Turn::Opponent), None];

impl OptimalCache {
    pub fn new() -> Self {
        Self {
            states: OptimalWinningStates::new(),
        }
    }

    pub fn load_from_disk(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let cache = fs::read(path.as_ref()).map_err(|e| e.to_string())?;
        if cache.len() % 4 != 0 {
            return Err("Malformed cache (expected sequence of 32-bit values).".to_string());
        }

        let mut remaining;
        let mut it = cache.as_slice().chunks_exact(4).map(|chunk| {
            let arr = chunk.try_into().unwrap();
            u32::from_le_bytes(arr)
        });

        for t in &OPTIMAL_SERIALIZATION_ORDER {
            remaining = it.next().ok_or_else(|| "Expected a number".to_string())?;
            for _ in 0..remaining {
                let num = it.next().ok_or_else(|| "Expected a number".to_string())?;
                let state = unsafe { std::mem::transmute(num) }; // TODO: might be done without unsafe
                self.states.entry(*t).or_default().insert(state);
            }
        }

        if !matches!(it.next(), None) {
            return Err("Unknown trailing data".to_string());
        }

        Ok(())
    }

    pub fn save_to_disk(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let mut buf = vec![];

        let empty_hashset = HashSet::new();
        let mut write_u32_to_buf = |num: u32| buf.extend_from_slice(&num.to_le_bytes());
        for t in &OPTIMAL_SERIALIZATION_ORDER {
            let elems = self.states.get(t).unwrap_or(&empty_hashset);
            write_u32_to_buf(elems.len().try_into().unwrap());
            for e in elems {
                let num: u32 = unsafe { std::mem::transmute(*e) };
                write_u32_to_buf(num);
            }
        }

        fs::write(path.as_ref(), &buf).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        for (_, v) in self.states.iter() {
            len += v.len();
        }
        len
    }

    // first option - if in cache
    // second option - None - draw, Some(t) - t wins
    fn get_state_winningness(&self, state: State) -> Option<Option<Turn>> {
        for t in [Some(Turn::Player), Some(Turn::Opponent), None] {
            if self
                .states
                .get(&t)
                .map_or(false, |col| col.contains(&state))
            {
                return Some(t);
            }
        }

        None
    }
}
