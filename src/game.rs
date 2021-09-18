use crate::{state::VerboseState, strategy::Optimal};
use console_engine::{ConsoleEngine, KeyCode};

pub struct Game {
    state: VerboseState,
    // strategy: Optimal,
    engine: ConsoleEngine,

    needs_redrawing: bool,
}

impl Game {
    const SCREEN_WIDTH: u32 = 30;
    const SCREEN_HEIGHT: u32 = 12;
    const SCREEN_FPS: u32 = 30;

    pub fn new(state: VerboseState) -> Game {
        // let strategy = Optimal::new(&state);
        let engine = console_engine::ConsoleEngine::init(
            Self::SCREEN_WIDTH,
            Self::SCREEN_HEIGHT,
            Self::SCREEN_FPS,
        )
        .unwrap();

        Game {
            state,
            // strategy,
            engine,

            needs_redrawing: true,
        }
    }

    pub fn run(mut self) {
        loop {
            self.engine.wait_frame();
            if self.engine.is_key_pressed(KeyCode::Char('q')) {
                break;
            }

            self.run_logic();

            if self.needs_redrawing {
                self.redraw();
                self.needs_redrawing = false;
            }
        }
    }

    fn run_logic(&mut self) {
        // todo!()
    }

    fn redraw(&mut self) {
        self.engine.clear_screen();

        // let mut buf = [' '; 24];
        // let

        self.print_centered(1, "Opponent");
        self.print_centered(2, "cards");
        // 4-selector
        self.print_centered(5, "cards");
        // 6 or 7 - selector
        self.print_centered(8, "cards");
        self.print_centered(9, "You");
        // 9 strategy state

        self.engine.draw();
    }

    fn print_centered(&mut self, line: i32, s: &str) -> i32 {
        let start_col = (Self::SCREEN_WIDTH as i32 - s.len() as i32) / 2;
        self.engine.print(start_col, line, s);
        start_col
    }

    // fn print_hand
}
