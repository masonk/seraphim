use std::{
    io,
    sync::{
        atomic::{
            AtomicBool, 
            Ordering
        },
        Arc
    },
};

use crate::{
    game,
    game::{GameStatus},
    inference, 
    search, 
    error::Result
};

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Ply<State> {
    state: State,
    action: usize,
}

pub struct InteractiveSession<Inference, State, Game, Player>
where
    Player: game::Player,
    State: game::GameState + std::fmt::Display,
    Game: game::Game<State = State> + crate::game::AsciiInteractive<Player = Player>,
    Inference: inference::Inference
{
    debug: bool,
    searcher: search::SearchTree<Inference, State, Game>,
    options: search::SearchTreeOptions
}

impl<Inference, State, Game, Player> InteractiveSession<Inference, State, Game, Player>
where
    Player: game::Player,
    Inference: inference::Inference,
    State: game::GameState + std::fmt::Display,
    Game: game::Game<State = State> + crate::game::AsciiInteractive<Player = Player>,
{
    pub fn new(
        inference: Inference,
        game: Game,
        root: State
    ) -> Self {
        Self::new_with_options(inference, game, root, search::SearchTreeOptions::default())
    }

    pub fn new_with_options(
        inference: Inference,
        game: Game,
        root: State,
        options: search::SearchTreeOptions,
    ) -> Self {
        let _current_state = root.clone();
        let searcher = search::SearchTree::init_with_options(inference, game, options.clone());
        InteractiveSession {
            debug: true,
            searcher,
            options
        }
    }

    fn prompt_next_action_debug_info(
        debug_info: &search::SearchResultsDebugInfo,
    ) -> usize {
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input);
            let trimmed = input.trim();
            if trimmed == "q" {
                panic!("quit");
            }
            if trimmed == "" {
                return debug_info.results.selection;
            }
            let parse = trimmed.parse::<usize>();
            if let Ok(selection) = parse {
                return debug_info.candidates[selection].action;
            } else {
                if let Err(e) = parse {
                    println!("{:?}", e);
                }
            }
            println!("Expected an integer, empty line, or \"q\"");
        }
    }

    pub fn start_debug(&mut self, running: Arc<AtomicBool>) -> Result<()> {
        println!("Press enter to play the computer's move. Enter an integer of different action to play that. q quits.");
        'outer: while running.load(Ordering::SeqCst) {
            let status = self.searcher.status();
            match status {
                GameStatus::InProgress => {
                    println!("{:?}", self.searcher.current_state_ref());
                    println!("{}", self.to_play());
                        
                    let debug = &self.searcher.read_debug()?;
                    for (i, info) in debug.candidates.iter().enumerate() {
                        println!("{}", self.show_action_info(i, info));
                    }
                    let hot = if debug.hot { "HOT" } else { "COLD " };
                    println!("Temperature is {}", hot);
                    println!("Computer would play {}.", debug.results.selection);
                    let sec = (debug.time.as_secs() as f64) + (debug.time.subsec_nanos() as f64 / 1000_000_000.0);
                    let per_sec =  self.options.readouts as f64 / sec;
                    println!("{:.3}s ({:.2} readouts / s)", sec, per_sec);

                    let next_action = Self::prompt_next_action_debug_info(&debug);

                    self.searcher.apply(next_action);
                },
                GameStatus::LastPlayerLost => {
                    println!("{:?} won", self.to_play());
                    break;
                },
                GameStatus::LastPlayerWon => {
                    println!("{:?} lost", self.to_play());
                    break;
                },
                GameStatus::Draw => {
                    println!("Draw.");
                    break;
                },
                rest @ _ => {
                    println!("{:?}", rest);
                    break;
                }
            }
        }
        Ok(())
    }
    pub fn start_game(&mut self, running: Arc<AtomicBool>) -> Result<()> {
        'outer: while running.load(Ordering::SeqCst) {
            let status = self.searcher.status();

            match status {
                GameStatus::InProgress => {
                    println!("{}", self.searcher.current_state_ref());
                    println!("{}", self.to_play());

                    match self.to_play().humanity() {
                        crate::game::Humanity::Human => {
                            for i in 0..self.searcher.action_count() {
                                println!("{}", i);
                            }

                            let next_action = self.get_next_action_interactive(&running);
                            self.searcher.apply(next_action);
                        },
                        crate::game::Humanity::Computer => {
                            let debug = &self.searcher.read_debug()?;
                            self.searcher.apply_search_results(&debug.results);
                            let prob_sum: f32 = debug.results.results.iter().sum();
                            if prob_sum < 0.0 {
                                println!(
                                    "missing net probability {} (ascribed to illegal moves?)",
                                    1.0 - prob_sum
                                );
                            }
                        }
                    }
                },
                GameStatus::LastPlayerLost => {
                    println!("{:?} won", self.to_play());
                    break;
                },
                GameStatus::LastPlayerWon => {
                    println!("{:?} lost", self.to_play());
                    break;
                },
                GameStatus::Draw => {
                    println!("Draw.");
                    break;
                },
                rest @ _ => {
                    println!("{:?}", rest);
                    break;
                }
            }
        }
        Ok(())
    }
    fn show_action_info(
        &self,
        idx: usize,
        info: &search::CandidateActionDebugInformation,
    ) -> String {
        format!("[{}] action: {} effective: {:0>6.4} search: {:>6.4} samples: {:>5} value: {:0>7.5} exploration_stimulus: {:0>7.5} total samples: {:>5}", 
        idx, 
        info.action, 
        info.prior,
        info.posterior, 
        info.visits_in_last_read, 
        info.average_value,
        info.exploration_stimulus,
        info.total_visits)
    }
    pub fn get_next_action_interactive(
        &mut self,
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
    pub fn to_play(&self) -> &Player {
        self.searcher.game_ref().to_play(self.searcher.current_state_ref())
    }
}
