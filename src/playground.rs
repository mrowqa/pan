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
    let _start_state = VerboseState::random();
    // let _optimal = Optimal::new(&start_state);
}

pub fn engine_example() {
    use console_engine::pixel;
    use console_engine::Color;
    use console_engine::KeyCode;

    let mut engine = console_engine::ConsoleEngine::init(20, 10, 3).unwrap();
    let value = 14;
    // main loop, be aware that you'll have to break it because ctrl+C is captured
    loop {
        engine.wait_frame(); // wait for next frame + capture inputs
        engine.clear_screen(); // reset the screen

        engine.line(0, 0, 19, 9, pixel::pxl('#')); // draw a line of '#' from [0,0] to [19,9]
        engine.print(0, 4, format!("Result: {}", value).as_str()); // prints some value at [0,4]

        engine.set_pxl(4, 0, pixel::pxl_fg('O', Color::Cyan)); // write a majestic cyan 'O' at [4,0]

        if engine.is_key_pressed(KeyCode::Char('q')) {
            // if the user presses 'q' :
            break; // exits app
        }

        engine.draw(); // draw the screen
    }
}
