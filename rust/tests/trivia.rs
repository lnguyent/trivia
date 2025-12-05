use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use trivia::{Category, Game, GameObserver};

struct Spy {
    count_go_to_penalty_box: HashMap<String, usize>,
    count_leave_penalty_box: HashMap<String, usize>,
    rolls: Vec<(String, usize)>,
    moves: Vec<(String, usize)>,
    wins: Vec<(String, usize)>,
}
impl Default for Spy {
    fn default() -> Self {
        Spy {
            count_go_to_penalty_box: HashMap::new(),
            count_leave_penalty_box: HashMap::new(),
            rolls: Vec::new(),
            moves: Vec::new(),
            wins: Vec::new(),
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
    fn on_roll(&mut self, player: &str, roll: usize) {
        self.spy.borrow_mut().rolls.push((player.to_string(), roll));
    }
    fn on_move(&mut self, _player: &str, _new_position: usize) {
        self.spy
            .borrow_mut()
            .moves
            .push((_player.to_string(), _new_position));
    }
    fn on_ask_question(&mut self, _category: Category, _question: Option<String>) {}
    fn on_win_point(&mut self, player: &str, new_score: usize) {
        self.spy
            .borrow_mut()
            .wins
            .push((player.to_string(), new_score));
    }
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
fn test_when_not_in_penalty_box_then_move() {
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new_with_observer(Box::new(spy_wrapper));

    game.add("Dave".to_string());
    game.roll(4); // move

    assert_eq!(spy.borrow().moves.len(), 1);
    assert_eq!(spy.borrow().moves[0].1, 4);
}

#[test]
fn test_when_leaving_penalty_box_then_not_in_penalty_box_anymore() {
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
#[test]
fn test_when_in_penalty_box_and_roll_odd_then_leave_penalty_box() {
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new_with_observer(Box::new(spy_wrapper));

    game.add("Eve".to_string());
    game.roll(1); // move
    game.wrong_answer(); // goes to penalty box
    game.roll(5); // leaves penalty box

    assert_eq!(spy.borrow().count_leave_penalty_box.get("Eve").unwrap(), &1);
}

#[test]
fn test_when_in_penalty_box_and_leaving_then_move() {
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new_with_observer(Box::new(spy_wrapper));

    game.add("Carol".to_string());
    game.roll(1); // move
    game.wrong_answer(); // goes to penalty box
    game.roll(3); // leaves penalty box and moves

    assert_eq!(spy.borrow().moves.len(), 2);
    assert_eq!(spy.borrow().moves[0].1, 1);
    assert_eq!(spy.borrow().moves[1].1, 4);
}

#[test]
fn test_when_answer_correctly_then_win_one_point() {
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new_with_observer(Box::new(spy_wrapper));

    game.add("Frank".to_string());
    game.roll(2); // move
    game.was_correctly_answered(); // wins one point
    game.roll(2); // move
    game.was_correctly_answered(); // wins another point

    assert_eq!(spy.borrow().wins.len(), 2);
    assert_eq!(spy.borrow().wins[0].0, "Frank");
    assert_eq!(spy.borrow().wins[0].1, 1);
    assert_eq!(spy.borrow().wins[1].0, "Frank");
    assert_eq!(spy.borrow().wins[1].1, 2);
}

#[test]
fn test_when_user_has_six_points_then_wins() {
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new_with_observer(Box::new(spy_wrapper));

    game.add("Grace".to_string());
    let mut not_a_winner = true;
    while not_a_winner {
        game.roll(3);
        not_a_winner = game.was_correctly_answered();
    }

    assert_eq!(spy.borrow().wins.len(), 6);
    assert_eq!(spy.borrow().wins[5].0, "Grace");
    assert_eq!(spy.borrow().wins[5].1, 6);
}

#[test]
fn test_on_roll_is_called() {
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new_with_observer(Box::new(spy_wrapper));

    game.add("Heidi".to_string());
    game.add("Ivan".to_string());
    game.roll(4);
    game.was_correctly_answered();
    game.roll(2);
    game.wrong_answer();
    game.roll(3);
    game.was_correctly_answered();
    game.roll(6);

    assert_eq!(spy.borrow().rolls.len(), 4);
    assert_eq!(spy.borrow().rolls[0].0, "Heidi");
    assert_eq!(spy.borrow().rolls[0].1, 4);
    assert_eq!(spy.borrow().rolls[1].0, "Ivan");
    assert_eq!(spy.borrow().rolls[1].1, 2);
    assert_eq!(spy.borrow().rolls[2].0, "Heidi");
    assert_eq!(spy.borrow().rolls[2].1, 3);
    assert_eq!(spy.borrow().rolls[3].0, "Ivan");
    assert_eq!(spy.borrow().rolls[3].1, 6);
}
