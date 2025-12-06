use std::collections::HashMap;
use std::fmt;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

const MAX_PLAYERS: usize = 6;
const NUM_PLACES_PER_CATEGORY: usize = 3;
const NUM_CARDS_PER_CATEGORY: usize = 50;
const TARGET_SCORE: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dice {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let category_str = match self {
            Dice::One => "1",
            Dice::Two => "2",
            Dice::Three => "3",
            Dice::Four => "4",
            Dice::Five => "5",
            Dice::Six => "6",
        };
        write!(f, "{}", category_str)
    }
}

impl Into<usize> for &Dice {
    fn into(self) -> usize {
        match self {
            Dice::One => 1,
            Dice::Two => 2,
            Dice::Three => 3,
            Dice::Four => 4,
            Dice::Five => 5,
            Dice::Six => 6,
        }
    }
}

impl From<i32> for Dice {
    fn from(value: i32) -> Self {
        match value {
            1 => Dice::One,
            2 => Dice::Two,
            3 => Dice::Three,
            4 => Dice::Four,
            5 => Dice::Five,
            6 => Dice::Six,
            _ => panic!("Invalid dice value"),
        }
    }
}
impl From<usize> for Dice {
    fn from(value: usize) -> Self {
        match value {
            1 => Dice::One,
            2 => Dice::Two,
            3 => Dice::Three,
            4 => Dice::Four,
            5 => Dice::Five,
            6 => Dice::Six,
            _ => panic!("Invalid dice value"),
        }
    }
}

impl Dice {
    pub fn is_odd(&self) -> bool {
        let value: usize = self.into();
        value % 2 == 1
    }
}

#[derive(Clone, Debug, EnumCountMacro, EnumIter, Eq, PartialEq, Hash)]
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
    fn on_roll(&mut self, player: &str, roll: Dice);
    fn on_move(&mut self, player: &str, new_position: usize);
    fn on_ask_question(&mut self, category: Category);
    fn on_win_point(&mut self, player: &str, new_score: usize);
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
    fn on_roll(&mut self, player: &str, roll: Dice) {
        println!("{} is current player", player);
        println!("They have rolled a {}", roll);
    }
    fn on_move(&mut self, player: &str, new_position: usize) {
        println!("{0} 's new location is {1}", player, new_position);
    }
    fn on_ask_question(&mut self, category: Category) {
        println!("The category is {}", category);
    }
    fn on_win_point(&mut self, player: &str, new_score: usize) {
        println!("Answer was correct!!!!");
        println!("{0} now has {1} Gold Coins.", player, new_score);
    }
    fn on_go_to_penalty_box(&mut self, player: &str) {
        println!("Question was incorrectly answered");
        println!("{} was sent to the penalty box", player);
    }
    fn on_leave_penalty_box(&mut self, player: &str) {
        println!("{} is getting out of the penalty box", player);
    }
    fn on_stay_in_penalty_box(&mut self, player: &str) {
        println!("{} is not getting out of the penalty box", player);
    }
}

// Command pattern for cards.
// I think it's more readable to name it "Card" rather than "CarfCommand"
// and to name the method "ask_question" rather than "execute".
pub trait Card {
    fn ask_question(&self) -> bool;
}

pub trait Deck {
    fn take_card(&mut self, category: Category) -> Option<Box<dyn Card>>;
}

struct DummyCard {
    question: String,
}

impl DummyCard {
    pub fn new(question: String) -> Self {
        DummyCard { question }
    }
}

impl Card for DummyCard {
    fn ask_question(&self) -> bool {
        println!("{:?}", self.question);
        true
    }
}

struct DummyDeck {
    cards: HashMap<Category, Vec<DummyCard>>,
}

impl Default for DummyDeck {
    fn default() -> Self {
        let mut cards = HashMap::new();
        for index in 0..NUM_CARDS_PER_CATEGORY {
            for category in Category::iter() {
                cards
                    .entry(category.clone())
                    .or_insert_with(Vec::new)
                    .push(DummyCard::new(
                        category.to_string() + " Question " + &index.to_string(),
                    ));
            }
        }
        DummyDeck { cards }
    }
}

impl Deck for DummyDeck {
    fn take_card(&mut self, category: Category) -> Option<Box<dyn Card>> {
        let card = self.cards.get_mut(&category).and_then(|cards| cards.pop());
        match card {
            Some(c) => Some(Box::new(c)),
            None => None,
        }
    }
}

struct AskQuestionResult {
    pub game_must_go_on: bool,
    pub answered_correctly: bool,
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

    deck: Box<dyn Deck>,
    observer: Box<dyn GameObserver>,
}

impl Default for Game {
    fn default() -> Game {
        Game::new(
            Box::new(DummyDeck::default()),
            Box::new(PrintBasedGameObserver::default()),
        )
    }
}

impl Game {
    pub fn new(deck: Box<dyn Deck>, observer: Box<dyn GameObserver>) -> Self {
        Game {
            players: vec![],
            places: [0; MAX_PLAYERS],
            purses: [0; MAX_PLAYERS],
            in_penaltybox: [false; MAX_PLAYERS],
            current_player: 0,
            is_getting_out_of_penaltybox: false,
            categories: Category::iter().collect(),
            num_places: Category::COUNT * NUM_PLACES_PER_CATEGORY,
            deck: deck,
            observer: observer,
        }
    }

    pub fn add(&mut self, player_name: String) {
        let l_player = player_name.clone();
        self.players.push(player_name);
        self.places[self.num_players()] = 0;
        self.purses[self.num_players()] = 0;
        self.in_penaltybox[self.num_players()] = false;
        self.observer.on_user_added(l_player.as_str());
    }

    pub fn roll(&mut self, roll: Dice) -> bool {
        self.observer
            .on_roll(self.players[self.current_player].as_str(), roll.clone());
        if self.in_penaltybox[self.current_player] {
            if (&roll).is_odd() {
                self.leave_penalty_box();
            } else {
                self.stay_in_penalty_box();
                return true;
            }
        }
        self.move_forward((&roll).into());
        let AskQuestionResult {
            game_must_go_on,
            answered_correctly,
        } = self.ask_question();
        if !game_must_go_on {
            return false;
        }
        if answered_correctly {
            return self.correct_answer();
        } else {
            return self.wrong_answer();
        }
    }

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

    fn ask_question(&mut self) -> AskQuestionResult {
        let category = self.current_category();
        self.observer.on_ask_question(category.clone());
        if let Some(card) = self.deck.take_card(category) {
            AskQuestionResult {
                game_must_go_on: true,
                answered_correctly: card.ask_question(),
            }
        } else {
            AskQuestionResult {
                game_must_go_on: false,
                answered_correctly: false,
            }
        }
    }

    fn wrong_answer(&mut self) -> bool {
        self.go_to_penalty_box();
        self.change_player();
        true
    }

    fn correct_answer(&mut self) -> bool {
        self.win_one_point();
        let game_must_go_on: bool = !self.did_player_win();
        self.change_player();
        game_must_go_on
    }

    fn leave_penalty_box(&mut self) {
        self.is_getting_out_of_penaltybox = true;
        self.in_penaltybox[self.current_player] = false;
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

    fn num_players(&self) -> usize {
        self.players.len()
    }

    fn did_player_win(&self) -> bool {
        self.purses[self.current_player] == TARGET_SCORE
    }

    fn current_category(&self) -> Category {
        self.categories[self.places[self.current_player] % self.categories.len()].clone()
    }

    fn change_player(&mut self) {
        self.current_player += 1;
        if self.current_player == self.players.len() {
            self.current_player = 0;
        }
    }

    fn win_one_point(&mut self) {
        self.purses[self.current_player] += 1;
        self.observer.on_win_point(
            self.players[self.current_player].as_str(),
            self.purses[self.current_player],
        );
    }
}
