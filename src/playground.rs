use crate::{
    state::{State, VerboseState},
    strategy::Optimal,
};
use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
};

pub fn foo() {
    fn visit(
        start_state: State,
        next_states: fn(&VerboseState) -> Vec<VerboseState>,
    ) -> HashSet<State> {
        let mut q = vec![start_state];
        let mut visited = HashSet::new();

        while let Some(s) = q.pop() {
            let following_states = next_states(&VerboseState::from(s));
            for s in following_states {
                let s = State::try_from(s).unwrap();
                if visited.contains(&s) {
                    continue;
                }
                visited.insert(s);
                q.push(s);
            }
        }

        visited
    }

    let start_state: State = VerboseState::random().try_into().unwrap();
    let visited_f = visit(start_state, VerboseState::following_states);
    let visited_p = visit(start_state, VerboseState::preceding_states);
    let intersection = visited_f.intersection(&visited_p).collect::<Vec<_>>();

    // Visited states: 15162474, 15137488, 15137466.
    // TODO: something is off.
    println!(
        "Visited states: {}, {}, {}.",
        visited_f.len(),
        visited_p.len(),
        intersection.len()
    );

    for s in visited_p.difference(&visited_f) {
        println!("{:?}", VerboseState::from(*s));
    }
}

pub fn bar() {
    let start_state = VerboseState::random();
    let _optimal = Optimal::new(&start_state);
}
