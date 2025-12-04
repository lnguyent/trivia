use std::collections::HashMap;
use std::fmt;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

const MAX_PLAYERS: usize = 6;
const NUM_PLACES_PER_CATEGORY: usize = 3;
const NUM_CARDS_PER_CATEGORY: usize = 50;
const TARGET_SCORE: usize = 6;

#[derive(Clone, EnumCountMacro, EnumIter, Eq, PartialEq, Hash)]
pub enum Category {
    Pop,
    Science,
    Sports,
    Rock,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let category_str = match self {
            Category::Pop => "Pop",
            Category::Science => "Science",
            Category::Sports => "Sports",
            Category::Rock => "Rock",
        };
        write!(f, "{}", category_str)
    }
}

pub trait GameObserver {
    fn on_user_added(&mut self, new_player_name: &str);
    fn on_roll(&mut self, player: &str, roll: usize);
    fn on_move(&mut self, player: &str, new_position: usize);
    fn on_ask_question(&mut self, category: Category, question: Option<String>);
    fn on_correct_answer(&mut self);
    fn on_win_point(&mut self, player: &str, new_score: usize);
    fn on_wrong_answer(&mut self);
    fn on_go_to_penalty_box(&mut self, player: &str);
    fn on_leave_penalty_box(&mut self, player: &str);
    fn on_stay_in_penalty_box(&mut self, player: &str);
}

struct PrintBasedGameObserver {
    num_players: usize,
}
impl Default for PrintBasedGameObserver {
    fn default() -> Self {
        PrintBasedGameObserver { num_players: 0 }
    }
}
impl GameObserver for PrintBasedGameObserver {
    fn on_user_added(&mut self, _new_player_name: &str) {
        self.num_players += 1;
        println!("{} was added", _new_player_name);
        println!("They are player number {}", self.num_players);
    }
    fn on_roll(&mut self, player: &str, roll: usize) {
        println!("{} is current player", player);
        println!("They have rolled a {}", roll);
    }
    fn on_move(&mut self, player: &str, new_position: usize) {
        println!("{0} 's new location is {1}", player, new_position);
    }
    fn on_ask_question(&mut self, category: Category, question: Option<String>) {
        println!("The category is {}", category);
        println!("{:?}", question.unwrap());
    }
    fn on_correct_answer(&mut self) {
        println!("Answer was correct!!!!");
    }
    fn on_win_point(&mut self, player: &str, new_score: usize) {
        println!("{0} now has {1} Gold Coins.", player, new_score);
    }
    fn on_wrong_answer(&mut self) {
        println!("Question was incorrectly answered");
    }
    fn on_go_to_penalty_box(&mut self, player: &str) {
        println!("{} was sent to the penalty box", player);
    }
    fn on_leave_penalty_box(&mut self, player: &str) {
        println!("{} is getting out of the penalty box", player);
    }
    fn on_stay_in_penalty_box(&mut self, player: &str) {
        println!("{} is not getting out of the penalty box", player);
    }
}

pub struct Game {
    players: Vec<String>,
    places: [usize; MAX_PLAYERS],
    purses: [usize; MAX_PLAYERS],
    in_penaltybox: [bool; MAX_PLAYERS],
    current_player: usize,
    is_getting_out_of_penaltybox: bool,

    categories: Vec<Category>,
    num_places: usize,

    questions: HashMap<Category, Vec<String>>,

    observer: Box<dyn GameObserver>,
}

impl Default for Game {
    fn default() -> Game {
        Game::new()
    }
}

impl Game {
    pub fn new() -> Game {
        let mut game = Game {
            players: vec![],
            places: [0; MAX_PLAYERS],
            purses: [0; MAX_PLAYERS],
            in_penaltybox: [false; MAX_PLAYERS],
            current_player: 0,
            is_getting_out_of_penaltybox: false,
            categories: Category::iter().collect(),
            num_places: Category::COUNT * NUM_PLACES_PER_CATEGORY,
            questions: HashMap::new(),
            observer: Box::new(PrintBasedGameObserver::default()),
        };
        for x in 0..NUM_CARDS_PER_CATEGORY {
            for category in Category::iter() {
                let question = game.create_question(category.clone(), x);
                game.questions
                    .entry(category)
                    .or_insert_with(Vec::new)
                    .push(question);
            }
        }
        game
    }

    fn how_many_players(&self) -> usize {
        self.players.len()
    }

    fn did_player_win(&self) -> bool {
        self.purses[self.current_player] != TARGET_SCORE
    }

    fn current_category(&self) -> Category {
        self.categories[self.places[self.current_player] % self.categories.len()].clone()
    }

    fn create_question(&self, category: Category, index: usize) -> String {
        category.to_string() + " Question " + &index.to_string()
    }

    pub fn add(&mut self, player_name: String) -> bool {
        let l_player = player_name.clone();
        self.players.push(player_name);
        self.places[self.how_many_players()] = 0;
        self.purses[self.how_many_players()] = 0;
        self.in_penaltybox[self.how_many_players()] = false;
        self.observer.on_user_added(l_player.as_str());
        true
    }

    pub fn wrong_answer(&mut self) -> bool {
        self.observer.on_wrong_answer();
        self.go_to_penalty_box();
        self.change_player();
        true
    }

    fn change_player(&mut self) {
        self.current_player += 1;
        if self.current_player == self.players.len() {
            self.current_player = 0;
        }
    }
}

impl Game {
    fn ask_question(&mut self) {
        let question = self
            .questions
            .get_mut(&self.current_category())
            .map(|questions| questions.pop().unwrap()); // FIXME: could be None!
        self.observer
            .on_ask_question(self.current_category(), question.clone());
    }
}

impl Game {
    fn move_forward(&mut self, roll: usize) {
        self.places[self.current_player] += roll;
        if self.places[self.current_player] > self.num_places - 1 {
            self.places[self.current_player] -= self.num_places;
        }
        self.observer.on_move(
            self.players[self.current_player].as_str(),
            self.places[self.current_player],
        );
    }

    fn leave_penalty_box(&mut self) {
        self.is_getting_out_of_penaltybox = true;
        self.observer
            .on_leave_penalty_box(self.players[self.current_player].as_str());
    }

    fn stay_in_penalty_box(&mut self) {
        self.observer
            .on_stay_in_penalty_box(self.players[self.current_player].as_str());
        self.is_getting_out_of_penaltybox = false;
    }

    fn go_to_penalty_box(&mut self) {
        self.observer
            .on_go_to_penalty_box(self.players[self.current_player].as_str());
        self.in_penaltybox[self.current_player] = true;
    }

    pub fn roll(&mut self, roll: usize) {
        self.observer
            .on_roll(self.players[self.current_player].as_str(), roll);
        if self.in_penaltybox[self.current_player] {
            if roll % 2 != 0 {
                self.leave_penalty_box();
            } else {
                self.stay_in_penalty_box();
            }
        }
        self.move_forward(roll);
        self.ask_question();
    }
}

impl Game {
    fn win_one_point(&mut self) {
        self.purses[self.current_player] += 1;
        self.observer.on_win_point(
            self.players[self.current_player].as_str(),
            self.purses[self.current_player],
        );
    }
    pub fn was_correctly_answered(&mut self) -> bool {
        self.observer.on_correct_answer();
        if self.in_penaltybox[self.current_player] && !self.is_getting_out_of_penaltybox {
            self.change_player();
            return true;
        }
        self.win_one_point();
        let winner: bool = self.did_player_win();
        self.change_player();
        winner
    }
}
