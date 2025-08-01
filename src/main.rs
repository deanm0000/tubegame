use rand::prelude::SliceRandom;
use rand::rng;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::io::{self, Write};
use std::num::{ParseFloatError, ParseIntError};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::{fmt, thread};
const CAPACITY: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Pink,
    Violet,
    Orange,
    LightGreen,
    Brown,
    DarkBlue,
}
#[derive(Clone, Debug)]
enum FromTo {
    From,
    To,
}

impl From<usize> for FromTo {
    fn from(s: usize) -> FromTo {
        match s {
            0 => FromTo::From,
            1 => FromTo::To,
            _ => panic!("must be 0 or 1"),
        }
    }
}
impl From<&usize> for FromTo {
    fn from(s: &usize) -> FromTo {
        match s {
            0 => FromTo::From,
            1 => FromTo::To,
            _ => panic!("must be 0 or 1"),
        }
    }
}

impl fmt::Display for FromTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FromTo::From => "from",
            FromTo::To => "to",
        };
        write!(f, "{s}")
    }
}

const COLORS: [Color; 12] = [
    Color::Red,
    Color::Green,
    Color::Blue,
    Color::Yellow,
    Color::Cyan,
    Color::Magenta,
    Color::Pink,
    Color::Violet,
    Color::Orange,
    Color::LightGreen,
    Color::Brown,
    Color::DarkBlue,
];
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Color::Red => "R",
            Color::Green => "G",
            Color::Blue => "B",
            Color::Yellow => "Y",
            Color::Cyan => "C",
            Color::Magenta => "M",
            Color::Pink => "P",
            Color::Violet => "V",
            Color::Orange => "O",
            Color::LightGreen => "L",
            Color::Brown => "N",
            Color::DarkBlue => "A",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Container {
    items: Vec<Color>, // bottom..top
}

impl Container {
    fn new() -> Self {
        Container { items: Vec::new() }
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn all_one_color(&self) -> bool {
        if self.items.len() == 0usize {
            return false;
        }
        let first_color = self.items.first().expect("checked len, why?");
        self.items.iter().all(|c| c == first_color)
    }
    fn is_uniform_full(&self) -> bool {
        self.items.len() == CAPACITY && self.items.iter().all(|&c| c == self.items[0])
    }
    fn is_full(&self) -> bool {
        self.items.len() == CAPACITY
    }

    fn top_color(&self) -> Option<Color> {
        self.items.last().copied()
    }

    fn push(&mut self, col: Color) {
        self.items.push(col);
    }

    fn pop(&mut self) -> Option<Color> {
        self.items.pop()
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
struct Game {
    tubes: Vec<Container>,
}

#[derive(Debug, Clone)]
struct Move {
    from: usize,
    to: usize,
}

#[derive(Debug, Clone)]
struct EnteredMove {
    from: usize,
    to: usize,
}

impl From<Vec<usize>> for EnteredMove {
    fn from(s: Vec<usize>) -> EnteredMove {
        EnteredMove {
            from: s[0],
            to: s[1],
        }
    }
}

impl Game {
    fn is_solved(&self) -> bool {
        self.tubes
            .iter()
            .all(|t| t.is_empty() || t.is_uniform_full())
    }

    /// Enumerate all legal moves from current state
    fn valid_moves(&self) -> Vec<Move> {
        let n = self.tubes.len();
        let mut out = Vec::new();
        for from_i in 0..n {
            if self.tubes[from_i].is_empty() || self.tubes[from_i].is_uniform_full() {
                continue;
            }
            let c = self.tubes[from_i]
                .top_color()
                .expect("checked not empty, shouldn't break");
            for to_i in 0..n {
                let uniform_color_to_empty =
                    self.tubes[from_i].all_one_color() && self.tubes[to_i].is_empty();
                if from_i == to_i || self.tubes[to_i].is_full() || uniform_color_to_empty {
                    continue;
                }
                if self.tubes[to_i].is_empty() || self.tubes[to_i].top_color() == Some(c) {
                    out.push(Move {
                        from: from_i,
                        to: to_i,
                    });
                }
            }
        }
        out
    }

    /// Apply a move (panics if illegal)
    fn apply(&mut self, mv: &Move) {
        loop {
            if self.tubes[mv.to].is_full() {
                break;
            };
            match (
                self.tubes[mv.from].top_color(),
                self.tubes[mv.to].top_color(),
            ) {
                (None, _) => break,
                (Some(from_color), None) => {
                    self.tubes[mv.from].pop();
                    self.tubes[mv.to].push(from_color);
                }
                (Some(from_color), Some(to_color)) => {
                    if from_color != to_color {
                        break;
                    };
                    self.tubes[mv.from].pop();
                    self.tubes[mv.to].push(from_color);
                }
            };
        }
    }

    fn display(&self) {
        for (i, tube) in self.tubes.iter().enumerate() {
            print!("Tube {i:>2}: |");
            for &c in &tube.items {
                print!("{c}");
            }
            for _ in tube.items.len()..CAPACITY {
                print!(" ");
            }
            println!("|");
        }
    }
}

fn generate_level(palette: &[Color], empty: usize) -> Game {
    // Start from solved
    let mut all_colors: Vec<Color> = Vec::with_capacity(palette.len() * CAPACITY);
    for col in palette {
        for _ in 0..CAPACITY {
            all_colors.push(*col);
        }
    }
    let mut my_rng = rng();
    all_colors.shuffle(&mut my_rng);
    let filled_tubes = palette.len();

    let mut tubes: Vec<Container> = (0..(filled_tubes + empty))
        .map(|_| Container::new())
        .collect();
    let mut j = 0;
    for col in all_colors {
        loop {
            if tubes[j].is_full() {
                j += 1;
            } else {
                break;
            }
        }
        tubes[j].push(col);
    }

    Game { tubes }
}

#[derive(Debug, Clone)]
struct FromPrevNode {
    parent: Game,
    mv: Move,
}
fn init_fixed() -> Game {
    let mut tubes: Vec<Container> = (0..14).map(|_| Container::new()).collect();
    tubes[0].items = vec![Color::Yellow, Color::Red, Color::Red, Color::Red];
    tubes[1].items = vec![
        Color::DarkBlue,
        Color::Yellow,
        Color::LightGreen,
        Color::LightGreen,
    ];
    tubes[2].items = vec![Color::Violet, Color::Pink, Color::Pink, Color::Pink];
    tubes[3].items = vec![Color::Green, Color::Orange, Color::Magenta, Color::Yellow];
    tubes[4].items = vec![
        Color::Brown,
        Color::LightGreen,
        Color::LightGreen,
        Color::DarkBlue,
    ];
    tubes[5].items = vec![Color::Yellow, Color::Red, Color::Magenta, Color::Brown];
    tubes[6].items = vec![Color::Blue, Color::Green, Color::Green, Color::Green];
    tubes[7].items = vec![Color::DarkBlue, Color::Violet, Color::Violet, Color::Violet];
    tubes[8].items = vec![Color::Brown, Color::Orange, Color::Orange, Color::Orange];
    tubes[9].items = vec![Color::Cyan, Color::DarkBlue, Color::Magenta, Color::Magenta];
    tubes[10].items = vec![Color::DarkBlue, Color::Pink, Color::Blue, Color::Blue];
    tubes[11].items = vec![Color::Brown, Color::Cyan, Color::Cyan, Color::Cyan];
    tubes[12].items = vec![];
    tubes[13].items = vec![];
    Game { tubes }
}

enum SolveRes {
    Timeout,
    NoSolution,
    Found(Vec<Move>),
}
fn solve_par(initial: &Game, max_sec_wait: u64) -> SolveRes {
    let n_threads = 50;
    let nodes: Arc<Mutex<HashMap<Game, FromPrevNode>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(n_threads);
    let game_move_queue: Arc<Mutex<Vec<(Game, Option<Move>)>>> =
        Arc::new(Mutex::new(vec![(initial.clone(), None)]));
    let (tx, rx) = mpsc::channel::<Vec<Move>>();
    let start = Instant::now();
    let timeout = Duration::from_secs(max_sec_wait);
    while start.elapsed() < timeout {
        if let Ok(done) = rx.try_recv() {
            handles.clear();
            let done = done.into_iter().rev().collect();
            return SolveRes::Found(done);
        }
        let game_move_queue_clone = Arc::clone(&game_move_queue);
        while handles.len() < n_threads && { game_move_queue_clone.lock().unwrap().len() } > 0 {
            let game_move_queue_clone = Arc::clone(&game_move_queue);
            let (next_game, next_move) = {
                let mut lock = game_move_queue_clone.lock().unwrap();
                lock.pop().expect("checked len>0, how?")
            };

            match next_move {
                None => {
                    let valid_moves = next_game.valid_moves();
                    for mv in valid_moves {
                        let mut lock = game_move_queue_clone.lock().unwrap();
                        lock.push((next_game.clone(), Some(mv)));
                    }
                }
                Some(valid_move) => {
                    let nodes_clone = Arc::clone(&nodes);
                    let tx_clone = tx.clone();
                    handles.push(thread::spawn(move || {
                        let mut child_game = next_game.clone();
                        child_game.apply(&valid_move);
                        if child_game.is_solved() {
                            let mut path = vec![valid_move];
                            let mut init_parent = next_game;
                            loop {
                                let nodes_clone = Arc::clone(&nodes_clone);
                                // let mut parent_vec: Vec<Game> = Vec::with_capacity(1);
                                let parent = {
                                    let lock = nodes_clone.lock().unwrap();
                                    let parent = lock.get(&init_parent);
                                    parent.map(|parent| parent.clone())
                                };
                                match parent {
                                    Some(parent) => {
                                        init_parent = parent.parent.clone();
                                        path.push(parent.mv);
                                    }
                                    None => {
                                        let _ = tx_clone.send(path);

                                        break;
                                    }
                                }
                            }
                        } else {
                            let mut add_to_queue = false;
                            {
                                let mut lock = nodes_clone.lock().unwrap();
                                match lock.entry(child_game.clone()) {
                                    Entry::Occupied(_) => {}
                                    Entry::Vacant(vac) => {
                                        vac.insert(FromPrevNode {
                                            parent: next_game,
                                            mv: valid_move,
                                        });
                                        add_to_queue = true;
                                    }
                                }
                            }
                            if add_to_queue {
                                let mut lock = game_move_queue_clone.lock().unwrap();
                                lock.push((child_game, None))
                            }
                        }
                    }))
                }
            }
        }
        while !handles.is_empty() {
            let mut dones: Vec<usize> = vec![];
            for (i, handle) in handles.iter().enumerate() {
                if handle.is_finished() {
                    dones.push(i);
                    break;
                }
            }
            // if any handles are to be removed then break out of this while
            let break_early = !dones.is_empty();
            for done in dones.into_iter().rev() {
                handles.remove(done);
            }
            if break_early {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        if { game_move_queue_clone.lock().unwrap().len() } == 0 && handles.is_empty() {
            return SolveRes::NoSolution;
        }
    }
    return SolveRes::Timeout;
}
fn main() {
    println!("Welcome to Water Sort Puzzle!");
    loop {
        let mut use_fixed = true;
        //use fixed or random game
        loop {
            print!("Use (f)ixed or (r)andom ");
            io::stdout().flush().unwrap();
            let mut entry = String::new();
            io::stdin().read_line(&mut entry).unwrap();
            let fr = entry.trim();
            match fr {
                "f" => {
                    break;
                }
                "r" => {
                    use_fixed = false;
                    break;
                }
                _ => {
                    println!("{fr} is invalid selection");
                }
            }
        }
        let mut game = match use_fixed {
            true => init_fixed(),
            false => {
                let palette: Vec<Color>;
                loop {
                    print!("How many colors ");
                    io::stdout().flush().unwrap();
                    let mut entry = String::new();
                    io::stdin().read_line(&mut entry).unwrap();
                    let num_colors: Result<usize, ParseIntError> = entry.trim().parse();
                    match num_colors {
                        Ok(n) => match n {
                            2..13 => {
                                palette = COLORS[0..n - 1].to_vec();
                                break;
                            }
                            _ => {
                                println!("can't do {n}");
                            }
                        },
                        Err(_) => {
                            println!("don't understand {entry}");
                        }
                    }
                }
                let empties;
                loop {
                    print!("How many empties ");
                    io::stdout().flush().unwrap();
                    let mut entry = String::new();
                    io::stdin().read_line(&mut entry).unwrap();
                    let res: Result<usize, ParseIntError> = entry.trim().parse();
                    match res {
                        Ok(n) => {
                            empties = n;
                            break;
                        }
                        Err(_) => {
                            println!("don't understand {entry}");
                        }
                    }
                }
                generate_level(&palette, empties)
            }
        };

        // pre-solve time

        let max_wait: u64;
        loop {
            print!("How many seconds to attempt presolve (0 means don't presolve) ");
            io::stdout().flush().unwrap();
            let mut entry = String::new();
            io::stdin().read_line(&mut entry).unwrap();
            let res: Result<f64, ParseFloatError> = entry.trim().parse();
            match res {
                Ok(n) => {
                    max_wait = n as u64;
                    break;
                }
                Err(_) => {
                    println!("don't understand {entry}");
                }
            }
        }
        match max_wait {
            0 => {}
            _ => {
                let solution = solve_par(&game, max_wait);
                match solution {
                    SolveRes::NoSolution => {
                        println!("no solution found");
                    }
                    SolveRes::Timeout => {
                        println!("solution couldn't be found in time");
                    }
                    SolveRes::Found(sol) => {
                        println!("is solvable {sol:?}");
                    }
                };
            }
        }

        loop {
            game.display();
            if game.is_solved() {
                println!("Congratulations—You solved it!");
                break;
            }
            print!("Enter move (from to): ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let parts: Vec<usize> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.len() != 2 {
                println!("Please enter two indices, e.g. `0 3`.");
                continue;
            }
            let mut do_continue = false;
            parts.iter().for_each(|n| {
                if n >= &game.tubes.len() {
                    let fromto: FromTo = n.into();
                    println!("The {fromto} index is out of bounds");
                    do_continue = true;
                }
            });
            if do_continue {
                continue;
            }
            let entered: EnteredMove = parts.into();
            if game.tubes[entered.from].is_empty() {
                println!("from tube is empty");
                continue;
            }
            let mv = Move {
                from: entered.from,
                to: entered.to,
            };
            game.apply(&mv);
        }
    }
}
