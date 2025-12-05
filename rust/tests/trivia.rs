use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use trivia::{Category, Game, GameObserver};

struct Spy {
    count_go_to_penalty_box: HashMap<String, usize>,
    count_leave_penalty_box: HashMap<String, usize>,
    moves: Vec<(String, usize)>,
}
impl Default for Spy {
    fn default() -> Self {
        Spy {
            count_go_to_penalty_box: HashMap::new(),
            count_leave_penalty_box: HashMap::new(),
            moves: Vec::new(),
        }
    }
}

// Required because of the orphan rule:
// cannot directly implement GameObserver for Rc<RefCell<Spy>>
// because neither GameObserver nor Rc are defined in this crate
struct SpyWrapper {
    spy: Rc<RefCell<Spy>>,
}

impl GameObserver for SpyWrapper {
    fn on_user_added(&mut self, _new_player_name: &str) {}
    fn on_roll(&mut self, _player: &str, _roll: usize) {}
    fn on_move(&mut self, _player: &str, _new_position: usize) {
        self.spy
            .borrow_mut()
            .moves
            .push((_player.to_string(), _new_position));
    }
    fn on_ask_question(&mut self, _category: Category, _question: Option<String>) {}
    fn on_correct_answer(&mut self) {}
    fn on_win_point(&mut self, _player: &str, _new_score: usize) {}
    fn on_wrong_answer(&mut self) {}
    fn on_go_to_penalty_box(&mut self, player: &str) {
        *self
            .spy
            .borrow_mut()
            .count_go_to_penalty_box
            .entry(player.to_string())
            .or_insert(0) += 1;
    }
    fn on_leave_penalty_box(&mut self, player: &str) {
        *self
            .spy
            .borrow_mut()
            .count_leave_penalty_box
            .entry(player.to_string())
            .or_insert(0) += 1;
    }
    fn on_stay_in_penalty_box(&mut self, _player: &str) {}
}

#[test]
fn test_when_leaving_penalty_box_then_not_in_penalty_box() {
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new_with_observer(Box::new(spy_wrapper));

    game.add("Alice".to_string());
    game.roll(1);
    game.wrong_answer(); // goes to penalty box
    game.roll(1); // leaves penalty box
    game.was_correctly_answered();
    game.roll(1); // already out of penalty box
    game.was_correctly_answered();

    assert_eq!(
        spy.borrow().count_leave_penalty_box.get("Alice").unwrap(),
        &1
    );
}

#[test]
fn test_when_in_penalty_box_then_do_not_move() {
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new_with_observer(Box::new(spy_wrapper));

    game.add("Bob".to_string());
    game.roll(1); // move
    game.wrong_answer(); // goes to penalty box
    game.roll(2); // stays in penalty box and do not move

    assert_eq!(spy.borrow().moves.len(), 1);
    assert_eq!(spy.borrow().moves[0].1, 1);
}
