// TODO: maybe lib.rs?

mod game;
// mod playground;
mod state;
mod strategy;

static CACHE_PATH: &str = "pan_cache.bin";

fn main() {
    let state = state::VerboseState::random();
    let mut cache = strategy::OptimalCache::new();

    println!("Trying to load cache if present.");
    if let Err(err) = cache.load_from_disk(CACHE_PATH) {
        eprintln!("Error while loading cache: {}", err)
    };

    println!("If cache is not built up, calculating strategy may take a few minutes.");
    game::Game::new(state, &mut cache).run();

    // In theory might be saved right after calculating the strategy, but it does not matter much.
    if let Err(err) = cache.save_to_disk(CACHE_PATH) {
        eprintln!("Error while saving cache: {}", err);
    }
}

// TODO:
// - displaying strategy state does not work (or maybe strategy does not work?)
