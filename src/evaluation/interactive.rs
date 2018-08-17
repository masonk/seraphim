use search;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Player {
    First,
    Second
}

impl Player {
    fn other(&self) -> Self {
        if self == &Player::First {
            Player::Second
        }
        else {
            Player::First
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum PlayerType {
    Computer,
    Human
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Ply<State, Action> {
    state: State,
    action: Action
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

pub struct InteractiveSession<State, Action, Expert> where 
Expert: search::GameExpert<State, Action>,
Action: ::std::cmp::PartialEq + ::std::hash::Hash + ::std::fmt::Debug,
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
    searcher: search::SearchTree<State, Action>
}

impl<State, Action, Expert> InteractiveSession<State, Action, Expert> where
Expert: search::GameExpert<State, Action>,
State: search::State,
Action: ::std::cmp::PartialEq + ::std::fmt::Display + ::std::hash::Hash + ::std::fmt::Debug,
{
    pub fn new(expert: Expert, root: State) -> Self {
        Self::new_with_options(expert, root, search::SearchTreeOptions::defaults())
    }

    pub fn new_with_options(expert: Expert, root: State, options: search::SearchTreeOptions) -> Self {
        let current_state = root.clone();
        let searcher = search::SearchTree::init_with_options(root.clone(), options);
        InteractiveSession {
            player1_type: PlayerType::Computer,
            player2_type: PlayerType::Human,
            next_player: Player::First,
            debug: true,
            expert,
            foo:  None,
            root,
            current_state,
            searcher
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
                self.println("", &format!("Next player has {} probability of winning", hypotheses.to_win));

                if self.next_player_type() == PlayerType::Human {
                    self.println(&self.show_available_actions(&hypotheses), &self.show_hypotheses(&hypotheses));
                    let next_action_selection = self.get_next_action_interactive(&hypotheses, &running);
                    let (next_action, _) = &hypotheses.legal_actions[next_action_selection];
                    self.searcher.apply(next_action);
                    self.current_state = self.expert.next(&self.current_state, next_action);
                }
                else {
                    let next_action = &self.searcher.read_and_apply(&mut self.expert);
                    self.println("", &self.show_hypotheses(&hypotheses));
                    self.current_state = self.expert.next(&self.current_state, next_action);
                }

                self.next_player = self.next_player.other();
                // self.push_history(Ply {
                //     state: current_state,
                //     action: next_action,
                // });

            } else {
                println!("{}", self.current_state);
                println!("{} won", self.show_next_player());
                println!("Game terminated with result {:?}", self.current_state.status());
                break;
            }
        }
    }
    fn push_history(&mut self, ply: Ply<Action, State>) {}
    pub fn get_next_action_interactive(&mut self, hypotheses: &search::Hypotheses<Action>, running: &Arc<AtomicBool>) -> usize {
        while running.load(Ordering::SeqCst) {
            let mut input = String::new();
            io::stdin().read_line(&mut input);
            let parse = input.trim().parse::<usize>();
            if let Ok(selection) = parse {
                return selection;
            }
            else {
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
        }
        else {
            self.player2_type
        }
    }
    pub fn show_next_player(&self) -> String {
        format!("{:?} ({:?}) to move", self.next_player, self.next_player_type())
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
        }
        else {
            if string.len() > 0 {
                println!("{}", string);
            }
        }
    }
}
