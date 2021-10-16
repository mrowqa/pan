use crate::{
    state::{CardsHand, Move, MoveDescription, Turn, VerboseState},
    strategy::{Optimal, OptimalCache, Strategy},
};
use console_engine::{pixel, Color, ConsoleEngine, KeyCode};
use std::cmp::max;

pub struct Game<'a> {
    state: VerboseState,
    strategy: Optimal<'a>,
    engine: ConsoleEngine,

    last_opponent_move: Option<MoveDescription>,
    player_moves: Vec<Move>,
    player_move_sel: Option<usize>,
    game_finished: bool,

    needs_redrawing: bool,
}

impl<'a> Game<'a> {
    const SCREEN_WIDTH: u32 = 30;
    const SCREEN_HEIGHT: u32 = 12;
    const SCREEN_FPS: u32 = 30;

    pub fn new(state: VerboseState, cache: &'a mut OptimalCache) -> Self {
        let strategy = Optimal::new_with_mut_cache(&state, cache);
        let engine = console_engine::ConsoleEngine::init(
            Self::SCREEN_WIDTH,
            Self::SCREEN_HEIGHT,
            Self::SCREEN_FPS,
        )
        .unwrap();
        let game_finished = state.is_game_finished();

        Game {
            state,
            strategy,
            engine,

            last_opponent_move: None,
            player_moves: vec![],
            player_move_sel: None,
            game_finished,

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
        if !self.game_finished && self.state.is_game_finished() {
            self.game_finished = true;
            self.needs_redrawing = true;
        }

        if !self.game_finished {
            if self.state.turn == Turn::Opponent {
                let mov = self
                    .strategy
                    .get_next_move(&self.state)
                    .expect("game not finished");
                self.last_opponent_move = Some(mov.desc);
                self.state = mov.state;
                self.needs_redrawing = true;
            } else if self.state.turn == Turn::Player {
                if self.player_moves.is_empty() {
                    self.player_moves = self.state.possible_moves();
                    assert!(!self.player_moves.is_empty());
                    self.player_move_sel = Some(0);
                    self.needs_redrawing = true;
                }

                let moves_cnt = self.player_moves.len();

                if self.engine.is_key_pressed(KeyCode::Left) {
                    self.player_move_sel = self.player_move_sel.map(|idx| (idx + 1) % moves_cnt);
                    self.needs_redrawing = true;
                }
                if self.engine.is_key_pressed(KeyCode::Right) {
                    self.player_move_sel = self
                        .player_move_sel
                        .map(|idx| (idx + moves_cnt - 1) % moves_cnt);
                    self.needs_redrawing = true;
                }
                if self.engine.is_key_pressed(KeyCode::Enter) {
                    self.state = self.player_moves[self.player_move_sel.unwrap()]
                        .state
                        .clone();
                    self.player_moves.clear();
                    self.player_move_sel = None;
                    self.needs_redrawing = true;
                }
            }
        }
    }

    fn redraw(&mut self) {
        self.engine.clear_screen();

        self.print_centered(1, "Opponent");
        self.print_hand(2, |s| &s.state.opponent_hand);
        let stack_coords = self.print_hand(5, |s| &s.state.table_stack);
        self.print_opponent_selector(4, stack_coords);
        self.print_player_selector_if_take(6, stack_coords);
        let player_coords = self.print_hand(8, |s| &s.state.player_hand);
        self.print_player_selector_if_plays_cards(7, player_coords);
        self.print_centered(9, "You");
        if self.game_finished {
            self.print_centered(10, "Game over");
        } else {
            self.print_strategy_state(10);
        }

        self.engine.draw();
    }

    fn print_centered(&mut self, line: i32, s: &str) -> (i32, i32) {
        let start_col = (Self::SCREEN_WIDTH as i32 - s.len() as i32) / 2;
        self.engine.print(start_col, line, s);
        (start_col, start_col + (s.len() as i32))
    }

    fn print_hand(&mut self, line: i32, get_hand: fn(&Self) -> &CardsHand) -> (i32, i32) {
        let mut hand_str = String::new(); // might be optimized
        if &self.state.table_stack == get_hand(self) {
            // Table stack nine is not taken into account.
            hand_str.push(CardsHand::IDX_TO_CHAR[CardsHand::CARD_TYPES - 1]);
        }
        for i in (0..CardsHand::CARD_TYPES).rev() {
            let hand = get_hand(self);
            for _ in 0..hand.cards[i] {
                hand_str.push(CardsHand::IDX_TO_CHAR[i]);
            }
        }
        self.print_centered(line, &hand_str)
    }

    fn print_opponent_selector(&mut self, line: i32, coords: (i32, i32)) {
        let end_col = coords.1;
        let pxl = pixel::pxl_fg('v', Color::Red);
        match self.last_opponent_move {
            None => (),
            Some(MoveDescription::PutAll(i)) => {
                let cards_put = CardsHand::card_idx_to_cnt(i) as i32;
                self.engine
                    .line(end_col - cards_put, line, end_col - 1, line, pxl)
            }
            Some(MoveDescription::PutSingle(_)) => self.engine.set_pxl(end_col - 1, line, pxl),
            Some(MoveDescription::Take) => self.engine.line(end_col, line, end_col + 2, line, pxl),
        }
    }

    fn print_player_selector_if_take(&mut self, line: i32, coords: (i32, i32)) {
        let (start_col, end_col) = coords;
        let pxl = pixel::pxl_fg('^', Color::Green);
        if let Some(mov_idx) = self.player_move_sel {
            if let MoveDescription::Take = self.player_moves[mov_idx].desc {
                let line_start = max(start_col + 1, end_col - 3);
                self.engine.line(line_start, line, end_col - 1, line, pxl);
            }
        }
    }

    fn print_player_selector_if_plays_cards(&mut self, line: i32, coords: (i32, i32)) {
        let start_col = coords.0;
        let pxl = pixel::pxl_fg('v', Color::Green);
        let calc_col = |idx: usize| {
            let mut col = start_col;
            for i in ((idx + 1)..(CardsHand::CARD_TYPES)).rev() {
                col += self.state.player_hand.cards[i] as i32;
            }
            col
        };
        if let Some(mov_idx) = self.player_move_sel {
            match self.player_moves[mov_idx].desc {
                MoveDescription::PutSingle(i) => {
                    let start_col = calc_col(i);
                    self.engine.set_pxl(start_col, line, pxl);
                }
                MoveDescription::PutAll(i) => {
                    let start_col = calc_col(i);
                    let end_col = start_col + self.state.player_hand.cards[i] as i32 - 1;
                    self.engine.line(start_col, line, end_col, line, pxl);
                }
                _ => (),
            }
        }
    }

    fn print_strategy_state(&mut self, line: i32) {
        let strategy_state = match self.strategy.get_winning_turn(&self.state) {
            None => "[S: Draw]",
            Some(Turn::Player) => "[S: Player]",
            Some(Turn::Opponent) => "[S: Opponent]",
        };
        self.engine.print(0, line, strategy_state);
    }
}
