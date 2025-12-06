extern crate rand;

use trivia::{Dice, Game};

fn main() {
    let mut game = Game::default();

    game.add("Chet".to_string());
    game.add("Pat".to_string());
    game.add("Sue".to_string());
    while game.roll(<usize as Into<Dice>>::into(rand::random::<usize>() % 6 + 1)) {}
}
