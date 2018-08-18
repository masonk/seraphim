use search;
use std::fmt::Write;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
        let searcher = search::SearchTree::init_with_options(root.clone(), options);
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
        }
    }

    pub fn current_state(&self) -> &State {
        &self.current_state
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

                    //     pub candidates: HashMap<Action, CandidateActionDebugInformation>, // how many visits does each child node have
                    //     pub results: SearchResultsInfo<Action>,
                    //     pub tempered: bool, // was tempering in effect for move selection
                    //     pub struct CandidateActionDebugInformation {
                    //     pub prior: f32,               // The naive probability that this move is the best
                    //     pub posterior: f32, // The improved probability that this move is the best after PUCT search
                    //     pub total_visits: u32, // how many times has this line of play been sampled, in total
                    //     pub visits_in_last_read: u32, // how many times was this line of play sampled in the most recent read
                    //     pub average_value: f32,       // The average value of taking this action Q(s, a) in the paper
                    //     pub total_value: f32,         // W(s, a) in the paper
                    let debug = &self.searcher.read_and_apply_debug(&mut self.expert);
                    let temp = if debug.hot { "hot" } else { "cold" };
                    println!("(temperature was {} for this search)", temp);
                    for (action, info) in debug.candidates.iter() {
                        println!("[{}] net: {} search: {} samples: {} value: {} total samples: {}", action, info.prior, info.posterior, info.visits_in_last_read, info.average_value, info.total_visits);
                    }

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