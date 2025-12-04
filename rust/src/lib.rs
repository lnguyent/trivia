use std::collections::HashMap;
use std::fmt;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

const MAX_PLAYERS: usize = 6;
const NUM_PLACES_PER_CATEGORY: usize = 3;
const NUM_CARDS_PER_CATEGORY: usize = 50;
const TARGET_SCORE: usize = 6;

#[derive(Clone, EnumCountMacro, EnumIter, Eq, PartialEq, Hash)]
enum Category {
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
        println!("{} was added", l_player);
        println!("They are player number {}", self.players.len());
        true
    }

    pub fn wrong_answer(&mut self) -> bool {
        println!("Question was incorrectly answered");
        println!(
            "{} was sent to the penalty box",
            self.players[self.current_player]
        );
        self.in_penaltybox[self.current_player] = true;
        self.current_player += 1;
        if self.current_player == self.players.len() {
            self.current_player = 0;
        }
        true
    }
}

impl Game {
    fn ask_question(&mut self) {
        self.questions
            .get_mut(&self.current_category())
            .map(|questions| {
                let question = questions.pop();
                println!("{:?}", question.unwrap());
            });
    }
}

impl Game {
    pub fn roll(&mut self, roll: usize) {
        println!("{} is current player", self.players[self.current_player]);
        println!("They have rolled a {}", roll);
        if self.in_penaltybox[self.current_player] {
            if roll % 2 != 0 {
                self.is_getting_out_of_penaltybox = true;
                println!(
                    "{} is getting out of the penalty box",
                    self.players[self.current_player]
                );
                self.places[self.current_player] += roll;
                if self.places[self.current_player] > self.num_places - 1 {
                    self.places[self.current_player] -= self.num_places;
                }
                println!(
                    "{0} 's new location is {1}",
                    self.players[self.current_player], self.places[self.current_player]
                );
                println!("The category is {}", self.current_category());
                self.ask_question();
            } else {
                println!(
                    "{} is not getting out of the penalty box",
                    self.players[self.current_player]
                );
                self.is_getting_out_of_penaltybox = false;
            }
        } else {
            self.places[self.current_player] += roll;
            if self.places[self.current_player] > self.num_places - 1 {
                self.places[self.current_player] -= self.num_places;
            }
            println!(
                "{0} 's new location is {1}",
                self.players[self.current_player], self.places[self.current_player]
            );
            println!("The category is {}", self.current_category());
            self.ask_question();
        }
    }
}

impl Game {
    pub fn was_correctly_answered(&mut self) -> bool {
        if self.in_penaltybox[self.current_player] {
            if self.is_getting_out_of_penaltybox {
                println!("Answer was correct!!!!");
                self.purses[self.current_player] += 1;
                println!(
                    "{0} now has {1} Gold Coins.",
                    self.players[self.current_player], self.purses[self.current_player]
                );
                let winner: bool = self.did_player_win();
                self.current_player += 1;
                if self.current_player == self.players.len() {
                    self.current_player = 0;
                }
                winner
            } else {
                self.current_player += 1;
                if self.current_player == self.players.len() {
                    self.current_player = 0;
                }
                true
            }
        } else {
            println!("Answer was correct!!!!");
            self.purses[self.current_player] += 1;
            println!(
                "{0} now has {1} Gold Coins.",
                self.players[self.current_player], self.purses[self.current_player]
            );
            let winner: bool = self.did_player_win();
            self.current_player += 1;
            if self.current_player == self.players.len() {
                self.current_player = 0;
            }
            winner
        }
    }
}
