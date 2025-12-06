extern crate rand;

use rand::{Rng, SeedableRng, StdRng};
use trivia::Game;

fn main() {
    let mut game = Game::default();

    // Use fixed seed for reproducible results
    let mut rng = StdRng::from_seed(&[1; 32]);

    game.add("Chet".to_string());
    game.add("Pat".to_string());
    game.add("Sue".to_string());
    while {
        let dice_value: i32 = rng.gen_range(1, 6); // strange cast for reproducibility
        game.roll(dice_value.try_into().unwrap())
    } {}
}
