use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use trivia::{Card, Category, Deck, Dice, Game, GameObserver};

struct Spy {
    count_go_to_penalty_box: HashMap<String, usize>,
    count_leave_penalty_box: HashMap<String, usize>,
    rolls: Vec<(String, Dice)>,
    moves: Vec<(String, usize)>,
    questions: Vec<Category>,
    wins: Vec<(String, usize)>,
}
impl Default for Spy {
    fn default() -> Self {
        Spy {
            count_go_to_penalty_box: HashMap::new(),
            count_leave_penalty_box: HashMap::new(),
            rolls: Vec::new(),
            moves: Vec::new(),
            questions: Vec::new(),
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
    fn on_roll(&mut self, player: &str, roll: Dice) {
        self.spy.borrow_mut().rolls.push((player.to_string(), roll));
    }
    fn on_move(&mut self, _player: &str, _new_position: usize) {
        self.spy
            .borrow_mut()
            .moves
            .push((_player.to_string(), _new_position));
    }
    fn on_ask_question(&mut self, category: Category) {
        self.spy.borrow_mut().questions.push(category);
    }
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

struct PredefinedAnswerCard {
    answer: bool,
}

impl PredefinedAnswerCard {
    pub fn new(answer: bool) -> Self {
        PredefinedAnswerCard { answer }
    }
}

impl Card for PredefinedAnswerCard {
    fn ask_question(&self) -> bool {
        self.answer
    }
}

struct PredefinedAnswersDeck {
    answers: Vec<bool>,
    cursor: usize,
}

impl PredefinedAnswersDeck {
    pub fn new(answers: Vec<bool>) -> Self {
        PredefinedAnswersDeck { answers, cursor: 0 }
    }

    pub fn always_true() -> Self {
        PredefinedAnswersDeck {
            answers: vec![true],
            cursor: 0,
        }
    }

    pub fn always_false() -> Self {
        PredefinedAnswersDeck {
            answers: vec![false],
            cursor: 0,
        }
    }
}

impl Deck for PredefinedAnswersDeck {
    fn take_card(&mut self, _category: Category) -> Option<Box<dyn Card>> {
        if self.cursor >= self.answers.len() {
            return None;
        }
        let answer = self.answers[self.cursor];
        self.cursor += 1;
        self.cursor %= self.answers.len();
        Some(Box::new(PredefinedAnswerCard::new(answer)))
    }
}

struct FixedSizeDeck {
    size: usize,
    cursor: usize,
}

impl FixedSizeDeck {
    pub fn new(size: usize) -> Self {
        FixedSizeDeck { size, cursor: 0 }
    }
}

impl Deck for FixedSizeDeck {
    fn take_card(&mut self, _category: Category) -> Option<Box<dyn Card>> {
        if self.cursor >= self.size {
            return None;
        }
        self.cursor += 1;
        Some(Box::new(PredefinedAnswerCard::new(true)))
    }
}

#[test]
fn test_when_not_in_penalty_box_then_move() {
    let deck = PredefinedAnswersDeck::always_true();
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Dave".to_string());
    game.roll(Dice::Four); // move

    assert_eq!(spy.borrow().moves.len(), 1);
    assert_eq!(spy.borrow().moves[0].1, 4);
}

#[test]
fn test_when_leaving_penalty_box_then_not_in_penalty_box_anymore() {
    let deck = PredefinedAnswersDeck::new(vec![false, true, true]);
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Alice".to_string());
    game.roll(Dice::One); // goes to penalty box
    game.roll(Dice::One); // leaves penalty box
    game.roll(Dice::One); // already out of penalty box

    assert_eq!(
        spy.borrow().count_leave_penalty_box.get("Alice").unwrap(),
        &1
    );
}

#[test]
fn test_when_in_penalty_box_then_do_not_move() {
    let deck = PredefinedAnswersDeck::always_false();
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Bob".to_string());
    game.roll(Dice::One); // goes to penalty box
    game.roll(Dice::Two); // stays in penalty box and do not move

    assert_eq!(spy.borrow().moves.len(), 1);
    assert_eq!(spy.borrow().moves[0].1, 1);
}
#[test]
fn test_when_in_penalty_box_and_roll_odd_then_leave_penalty_box() {
    let deck = PredefinedAnswersDeck::new(vec![false, true]);
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Eve".to_string());
    game.roll(Dice::One); // goes to penalty box
    game.roll(Dice::Five); // leaves penalty box

    assert_eq!(spy.borrow().count_leave_penalty_box.get("Eve").unwrap(), &1);
}

#[test]
fn test_when_in_penalty_box_and_leaving_then_move() {
    let deck = PredefinedAnswersDeck::new(vec![false, true]);
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Carol".to_string());
    game.roll(Dice::One); // goes to penalty box
    game.roll(Dice::Three); // leaves penalty box and moves

    assert_eq!(spy.borrow().moves.len(), 2);
    assert_eq!(spy.borrow().moves[0].1, 1);
    assert_eq!(spy.borrow().moves[1].1, 4);
}

#[test]
fn test_when_answer_correctly_then_win_one_point() {
    let deck = PredefinedAnswersDeck::always_true();
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Frank".to_string());
    game.roll(Dice::Two); // wins one point
    game.roll(Dice::Two); // wins one point

    assert_eq!(spy.borrow().wins.len(), 2);
    assert_eq!(spy.borrow().wins[0].0, "Frank");
    assert_eq!(spy.borrow().wins[0].1, 1);
    assert_eq!(spy.borrow().wins[1].0, "Frank");
    assert_eq!(spy.borrow().wins[1].1, 2);
}

#[test]
fn test_when_user_has_six_points_then_wins() {
    let deck = PredefinedAnswersDeck::always_true();
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Grace".to_string());
    let mut game_must_go_on = true;
    while game_must_go_on {
        game_must_go_on = game.roll(Dice::Three);
    }

    assert_eq!(spy.borrow().wins.len(), 6);
    assert_eq!(spy.borrow().wins[5].0, "Grace");
    assert_eq!(spy.borrow().wins[5].1, 6);
}

#[test]
fn test_on_roll_is_called() {
    let deck = PredefinedAnswersDeck::always_true();
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Heidi".to_string());
    game.add("Ivan".to_string());
    game.roll(Dice::Four);
    game.roll(Dice::Two);
    game.roll(Dice::Three);
    game.roll(Dice::Six);

    assert_eq!(spy.borrow().rolls.len(), 4);
    assert_eq!(spy.borrow().rolls[0].0, "Heidi");
    assert_eq!(spy.borrow().rolls[0].1, Dice::Four);
    assert_eq!(spy.borrow().rolls[1].0, "Ivan");
    assert_eq!(spy.borrow().rolls[1].1, Dice::Two);
    assert_eq!(spy.borrow().rolls[2].0, "Heidi");
    assert_eq!(spy.borrow().rolls[2].1, Dice::Three);
    assert_eq!(spy.borrow().rolls[3].0, "Ivan");
    assert_eq!(spy.borrow().rolls[3].1, Dice::Six);
}

#[test]
fn test_when_no_more_card_then_game_stops() {
    let deck = FixedSizeDeck::new(3); // only 3 cards available
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Judy".to_string());
    game.add("Karl".to_string());

    let mut game_must_go_on = true;
    let mut roll_count = 0;
    while game_must_go_on {
        game_must_go_on = game.roll(Dice::Two);
        roll_count += 1;
    }

    assert_eq!(roll_count, 4); // 4 rolls until no more cards
}

#[test]
fn test_full_game() {
    let rolls = vec![
        (Dice::Four, true),
        (Dice::Four, true),
        (Dice::Five, true),
        (Dice::Four, true),
        (Dice::Three, true),
        (Dice::Two, true),
        (Dice::Five, true),
        (Dice::Four, true),
        (Dice::One, true),
        (Dice::Five, true),
        (Dice::One, true),
        (Dice::Four, false),
        (Dice::Four, true),
        (Dice::One, true),
        (Dice::Five, true),
        (Dice::Five, true),
    ];
    let deck = PredefinedAnswersDeck::new(rolls.iter().map(|roll| roll.1).collect());
    let spy = Rc::new(RefCell::new(Spy::default()));
    let spy_wrapper = SpyWrapper { spy: spy.clone() };
    let mut game = Game::new(Box::new(deck), Box::new(spy_wrapper));

    game.add("Chet".to_string());
    game.add("Pat".to_string());
    game.add("Sue".to_string());

    let mut game_must_go_on: bool;
    for roll in rolls {
        game_must_go_on = game.roll(roll.0);
        if !game_must_go_on {
            break;
        }
    }

    assert_eq!(spy.borrow().rolls.len(), 16);
    assert_eq!(spy.borrow().rolls[0].0, "Chet");
    assert_eq!(spy.borrow().rolls[0].1, Dice::Four);
    assert_eq!(spy.borrow().rolls[1].0, "Pat");
    assert_eq!(spy.borrow().rolls[1].1, Dice::Four);
    assert_eq!(spy.borrow().rolls[2].0, "Sue");
    assert_eq!(spy.borrow().rolls[2].1, Dice::Five);
    // ... skipping some assertions for brevity ...
    assert_eq!(spy.borrow().rolls[14].0, "Sue");
    assert_eq!(spy.borrow().rolls[14].1, Dice::Five);
    assert_eq!(spy.borrow().rolls[15].0, "Chet");
    assert_eq!(spy.borrow().rolls[15].1, Dice::Five);

    assert_eq!(spy.borrow().moves.len(), 16);
    assert_eq!(spy.borrow().moves[0].0, "Chet");
    assert_eq!(spy.borrow().moves[0].1, 4);
    assert_eq!(spy.borrow().moves[1].0, "Pat");
    assert_eq!(spy.borrow().moves[1].1, 4);
    assert_eq!(spy.borrow().moves[2].0, "Sue");
    assert_eq!(spy.borrow().moves[2].1, 5);
    // ... skipping some assertions for brevity ...
    assert_eq!(spy.borrow().moves[14].0, "Sue");
    assert_eq!(spy.borrow().moves[14].1, 5);
    assert_eq!(spy.borrow().moves[15].0, "Chet");
    assert_eq!(spy.borrow().moves[15].1, 3);

    assert_eq!(spy.borrow().wins.len(), 15);
    assert_eq!(spy.borrow().wins[0].0, "Chet");
    assert_eq!(spy.borrow().wins[0].1, 1);
    assert_eq!(spy.borrow().wins[1].0, "Pat");
    assert_eq!(spy.borrow().wins[1].1, 1);
    assert_eq!(spy.borrow().wins[2].0, "Sue");
    assert_eq!(spy.borrow().wins[2].1, 1);
    // ... skipping some assertions for brevity ...
    assert_eq!(spy.borrow().wins[13].0, "Sue");
    assert_eq!(spy.borrow().wins[13].1, 4);
    assert_eq!(spy.borrow().wins[14].0, "Chet");
    assert_eq!(spy.borrow().wins[14].1, 6);

    assert_eq!(spy.borrow().questions.len(), 16);
    assert_eq!(spy.borrow().questions[0], Category::Pop);
    assert_eq!(spy.borrow().questions[1], Category::Pop);
    assert_eq!(spy.borrow().questions[2], Category::Science);
    // ... skipping some assertions for brevity ...
    assert_eq!(spy.borrow().questions[13], Category::Science);
    assert_eq!(spy.borrow().questions[14], Category::Science);
    assert_eq!(spy.borrow().questions[15], Category::Rock);

    assert!(spy.borrow().count_go_to_penalty_box.get("Chet").is_none());
    assert!(spy.borrow().count_leave_penalty_box.get("Chet").is_none());
    assert!(spy.borrow().count_go_to_penalty_box.get("Pat").is_none());
    assert!(spy.borrow().count_leave_penalty_box.get("Pat").is_none());
    assert_eq!(spy.borrow().count_go_to_penalty_box.get("Sue").unwrap(), &1);
    assert_eq!(spy.borrow().count_leave_penalty_box.get("Sue").unwrap(), &1);
}
