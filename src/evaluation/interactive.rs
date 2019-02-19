use search;
use std::fmt::Write;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Player {
    First,
    Second,
}

impl Player {
    fn other(&self) -> Self {
        if self == &Player::First {
            Player::Second
        } else {
            Player::First
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum PlayerType {
    Computer,
    Human,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Ply<State, Action> {
    state: State,
    action: Action,
}

// #[derive(Clone, Debug, PartialEq, Copy)]
// pub struct PlyAndThen<State, Action> {
//     ply: Ply<State, Action>,
//     continuation: Continuation<State, Action>
// }

// #[derive(Clone, Debug, PartialEq, Copy)]
// pub enum Continuation<State, Action> {
//     Of(PlyAndThen<State, Action>),
//     Terminated(search::GameStatus),
// }

// #[derive(Clone, Debug, PartialEq, Copy)]
// pub struct Node<State, Action> {
//     children: Vec<Continuation>,
//     parent: Option<Tree<State, Action>>
// }

// Implementing AsciiPlayable & AsciiDebuggable will enable seraphim to manage a text-based interactive game playing session
trait AsciiPlayable<State, Action> {
    fn move_picking_instructions() -> String; // How to read the action label grid and input a next move as a text string
    fn action_labeled_grid(&State) -> String; // A grid
    fn get_action_from_action_label(&str) -> Action; //
}
// Debuggability involves the display of debugging information, such as p values,
// in addition to the basics necessary for playing a game.
trait AsciiDebuggable<State, Action>: AsciiPlayable<State, Action> {}

// GenericPlayable and GenericDebuggable are for games that have their own means of displaying & accepting information from
// the user, such as by maintaining a connection to an external GUI process.
// Seraphim will manage an interactive game play session between computer and player and prompt GenericPlayable
// to display and collect certain information from the user to advance the game.
trait GenericPlayable<State, Action> {
    fn start_new_game();
}

// Debuggability involves the display of debugging information, such as p values,
// in addition to the basics necessary for playing a game.
trait GenericDebuggable<State, Action>: GenericPlayable<State, Action> {}

pub struct InteractiveSession<State, Action, Expert>
where
    Expert: search::GameExpert<State, Action>,
    Action: search::Action,
    State: search::State,
{
    player1_type: PlayerType,
    player2_type: PlayerType,
    next_player: Player,
    expert: Expert,
    foo: Option<Action>,
    // history: Node<State, Action>,
    // cursor: Option<Node<State, Action>>,
    root: State,
    current_state: State,
    debug: bool,
    searcher: search::SearchTree<State, Action>,
    options: search::SearchTreeOptions
}

impl<State, Action, Expert> InteractiveSession<State, Action, Expert>
where
    Expert: search::GameExpert<State, Action>,
    State: search::State,
    Action: search::Action,
{
    pub fn new(expert: Expert, root: State) -> Self {
        Self::new_with_options(expert, root, search::SearchTreeOptions::defaults())
    }

    pub fn new_with_options(
        expert: Expert,
        root: State,
        options: search::SearchTreeOptions,
    ) -> Self {
        let current_state = root.clone();
        let searcher = search::SearchTree::init_with_options(root.clone(), options.clone());
        InteractiveSession {
            player1_type: PlayerType::Computer,
            player2_type: PlayerType::Human,
            next_player: Player::First,
            debug: true,
            expert,
            foo: None,
            root,
            current_state,
            searcher,
            options
        }
    }

    pub fn current_state(&self) -> &State {
        &self.current_state
    }

    fn prompt_next_action_debug_info(
        debug_info: &search::SearchResultsDebugInfo<Action>,
    ) -> &Action {
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input);
            let trimmed = input.trim();
            if trimmed == "q" {
                panic!("quit");
            }
            if trimmed == "" {
                return &debug_info.results.selection;
            }
            let parse = trimmed.parse::<usize>();
            if let Ok(selection) = parse {
                return &debug_info.candidates[selection].action;
            } else {
                if let Err(e) = parse {
                    println!("{:?}", e);
                }
            }
            println!("Expected an integer, empty line, or \"q\"");
        }
    }

    pub fn start_debug(&mut self, running: Arc<AtomicBool>) {
        'outer: while running.load(Ordering::SeqCst) {
            if let search::GameStatus::InProgress = self.current_state.status() {
                println!("{}", self.current_state);
                println!("{:?}", self.next_player);
                    
                let debug = &self.searcher.read_debug(&mut self.expert);
                for (i, info) in debug.candidates.iter().enumerate() {
                    println!("{}", self.show_action_info(i, info));
                }
                let hot = if debug.hot { "HOT" } else { "COLD " };
                println!("Temperature is {}", hot);
                println!("Computer would play {}. Press enter to play that. Enter an integer of different action to play that. q quits.", debug.results.selection);

                let next_action = Self::prompt_next_action_debug_info(&debug);

                self.searcher.apply(&next_action);
                self.current_state = self.searcher.current_state_ref().clone();
                self.next_player = self.next_player.other();
            } else {
                println!("{}", self.current_state);
                if self.current_state.status() == search::GameStatus::LastPlayerWon {
                    println!("{:?} won", self.next_player.other());
                } else if self.current_state.status() == search::GameStatus::LastPlayerLost {
                    println!("{:?} won", self.next_player);
                } else {
                    println!("Draw.");
                }

                break;
            }
        }
    }
    pub fn start_game(&mut self, running: Arc<AtomicBool>) {
        'outer: while running.load(Ordering::SeqCst) {
            if let search::GameStatus::InProgress = self.current_state.status() {
                println!("{}", self.current_state);
                println!("{}", self.show_next_player());
                let hypotheses = self.expert.hypotheses(&self.current_state);
                self.println(
                    "",
                    &format!(
                        "Next player has {} probability of winning",
                        hypotheses.to_win
                    ),
                );

                if self.next_player_type() == PlayerType::Human {
                    self.println(
                        &self.show_available_actions(&hypotheses),
                        &self.show_hypotheses(&hypotheses),
                    );
                    let next_action_selection =
                        self.get_next_action_interactive(&hypotheses, &running);
                    let (next_action, _) = &hypotheses.legal_actions[next_action_selection];
                    self.searcher.apply(next_action);
                    self.current_state = self.expert.next(&self.current_state, next_action);
                } else {
                    //     pub candidates: Vec<CandidateActionDebugInformation>, // how many visits does each child node have
                    //     pub results: SearchResultsInfo<Action>,
                    //     pub tempered: bool, // was tempering in effect for move selection
                    //     pub struct CandidateActionDebugInformation {
                    //     pub prior: f32,               // The naive probability that this move is the best
                    //     pub posterior: f32, // The improved probability that this move is the best after PUCT search
                    //     pub total_visits: u32, // how many times has this line of play been sampled, in total
                    //     pub visits_in_last_read: u32, // how many times was this line of play sampled in the most recent read
                    //     pub average_value: f32,       // The average value of taking this action Q(s, a) in the paper
                    //     pub total_value: f32,         // W(s, a) in the paper
                    let debug = &self.searcher.read_debug(&mut self.expert);
                    self.searcher.apply_search_results(&debug.results);
                    if debug.hot {
                        println!("(temp was hot for this search)");
                    }

                    let prob_sum: f32 = debug.results.results.iter().sum();
                    if prob_sum < 0.0 {
                        println!(
                            "missing net probability {} (ascribed to illegal moves?)",
                            1.0 - prob_sum
                        );
                    }
                    for (i, info) in debug.candidates.iter().enumerate() {
                        println!("{}", self.show_action_info(i, info));
                    }
                    let sec = (debug.time.as_secs() as f64) + (debug.time.subsec_nanos() as f64 / 1000_000_000.0);
                    let per_sec =  self.options.readouts as f64 / sec;
                    println!("{:.3}s ({:.2} readouts / s)", sec, per_sec);
                    
                    self.current_state = self.expert
                        .next(&self.current_state, &debug.results.selection);
                }

                self.next_player = self.next_player.other();
            // self.push_history(Ply {
            //     state: current_state,
            //     action: next_action,
            // });
            } else {
                println!("{}", self.current_state);
                if self.current_state.status() == search::GameStatus::LastPlayerWon {
                    println!("{:?} won", self.next_player.other());
                } else if self.current_state.status() == search::GameStatus::LastPlayerLost {
                    println!("{:?} won", self.next_player);
                } else {
                    println!("Draw.");
                }

                break;
            }
        }
    }
    fn show_action_info(
        &self,
        idx: usize,
        info: &search::CandidateActionDebugInformation<Action>,
    ) -> String {
        format!("[{}] action: {} net: {:0>6.4} search: {:0>6.4} net (raw): {:0>6.4} loss: {:>+6.4} samples: {:>5} value: {:0>7.5} total samples: {:>5} exploration_stimulus: {:0>7.5}", 
        idx, 
        info.action, 
        info.prior, 
        info.posterior, 
        info.raw_prior,
        info.posterior - info.prior,
        info.visits_in_last_read, 
        info.average_value,
        info.total_visits, 
        info.exploration_stimulus)
    }
    fn push_history(&mut self, _ply: Ply<Action, State>) {}
    pub fn get_next_action_interactive(
        &mut self,
        _hypotheses: &search::Hypotheses<Action>,
        running: &Arc<AtomicBool>,
    ) -> usize {
        while running.load(Ordering::SeqCst) {
            let mut input = String::new();
            io::stdin().read_line(&mut input);
            let parse = input.trim().parse::<usize>();
            if let Ok(selection) = parse {
                return selection;
            } else {
                if let Err(e) = parse {
                    println!("{:?}", e);
                }
            }
            println!("need the number of a listed action");
        }
        panic!("quit");
    }
    pub fn next_player_type(&self) -> PlayerType {
        if self.next_player == Player::First {
            self.player1_type
        } else {
            self.player2_type
        }
    }
    pub fn show_next_player(&self) -> String {
        format!(
            "{:?} ({:?}) to move",
            self.next_player,
            self.next_player_type()
        )
    }
    pub fn show_available_actions(&self, hypotheses: &search::Hypotheses<Action>) -> String {
        let mut actions = String::new();
        let mut i = 0;
        for (ref action, _) in &hypotheses.legal_actions {
            write!(&mut actions, "[{}] {}\n", i, action);
            i += 1;
        }
        actions
    }
    pub fn show_hypotheses(&self, hypotheses: &search::Hypotheses<Action>) -> String {
        let mut buf = String::new();
        let mut i = 0;
        for (ref action, p) in &hypotheses.legal_actions {
            write!(&mut buf, "[{}] {} ({})\n", i, action, p);
            i += 1;
        }
        buf
    }

    fn println(&self, string: &str, debug_string: &str) {
        if self.debug {
            if debug_string.len() > 0 {
                println!("{}", debug_string);
            }
        } else {
            if string.len() > 0 {
                println!("{}", string);
            }
        }
    }
}
