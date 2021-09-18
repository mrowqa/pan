// TODO: maybe lib.rs?

mod game;
mod playground;
mod state;
mod strategy;

fn main() {
    let s = state::VerboseState::random();
    game::Game::new(s).run()
}
