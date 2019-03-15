//! This module implements informed Monte Carlo Tree Search. "Informed" means that the search is guided by an expert policy
//! that ascribes Bayesian prior probabilities to question of whether each possible next action is the best one.
//! Consumers of the Seraphim library are to implement the GameExpert trait, and pass an instance of GameExpert
//! to SearchTree.
use petgraph;
use rand::distributions::Dirichlet;
use rand::prelude::*;
use std::collections::HashMap;
use std::time;
use structopt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameStatus {
    InProgress,
    NullResult,
    Draw,
    LastPlayerWon,
    LastPlayerLost,
}
#[derive(Debug)]
pub struct Hypotheses<Action> {
    pub legal_actions: Vec<(Action, f32)>,
    pub to_win: f32,
}

#[derive(Debug)]
pub struct CandidateActionDebugInformation<Action>
where
    Action: self::Action,
{
    pub action: Action,
    pub prior: f32,               // Prior after scaling and noise
    pub raw_prior: f32,           // scaled to a probability distribution
    pub scaled_prior: f32,        // prior after scaling
    pub posterior: f32,           // The improved probability that this move is the best after PUCT search
    pub total_visits: usize,        // how many times has this line of play been sampled, in total
    pub wins: usize,
    pub losses: usize,
    pub visits_in_last_read: usize, // how many times was this line of play sampled in the most recent read
    pub average_value: f64,       // The average value of taking this action Q(s, a) in the paper
    pub exploration_stimulus: f64, // How badly does the search tree want to explore this action in the future?
                                   // The highest value of here is the node it would sample next if asked to perform more readouts.                                
}

#[derive(Debug)]
pub struct SearchResultsDebugInfo<Action>
where
    Action: self::Action,
{
    pub time: time::Duration, // How long it took to compute this move
    pub candidates: Vec<CandidateActionDebugInformation<Action>>,
    pub results: SearchResultsInfo<Action>,
    pub hot: bool, // Was this move chosen from a cold or hot sample. Hot introduces noise early in the game to ensure game variety.
}

#[derive(Debug)]
pub struct SearchResultsInfo<Action>
where
    Action: self::Action,
{
    pub results: Vec<f32>, // the improved probabitly estimates after searching. The result at a given index is the result for the Action that returned
    // that index from ActionIdx::index.
    pub selection: Action, // which move did the engine select
    pub application_token: ApplicationToken,
}

pub trait State:
    ::std::fmt::Display + ::std::hash::Hash + ::std::clone::Clone + ::std::fmt::Debug
{
    fn status(&self) -> GameStatus;
}

// search results will be returned as an array of
pub trait ActionIdx {
    fn index(&self) -> usize;
}

pub trait Action:
    ::std::cmp::PartialEq
    + ActionIdx
    + ::std::cmp::Eq
    + ::std::fmt::Display
    + ::std::hash::Hash
    + ::std::clone::Clone
    + ::std::fmt::Debug
{
}
// The expert that guides the MCTS search is abstracted by the GameExpert trait, which users of this library are to implement.
// The GameExpert knows the rules of the game it's playing, and also has bayesian prior beliefs about which moves are best to
// play and the probability that the next player will ultimately win the game. The search algorithm implemented by the SearchTree
// crucially depends on the accuracy of the expert's prior beliefs for its efficiency.
pub trait GameExpert<State, Action>
where
    State: self::State,
    Action: self::Action,
{
    // Hypotheses are a list of every legal next action, each paired with an unnormalized probablity of
    // of that action being the best one.
    //
    // Neural nets will typically ascribe a some amount of probability to non-legal actions.
    // There is no need to redistribute that probablity to the legal actions. Seraphim will
    // automatically redistribute missing probability while maintaining the relative likelihood of move
    // selection. (If you want to do a different distribution, simply apply it before returning the hypotheses.)
    //
    // The total raw probability of legal actions in a nonterminal state must be > 0.
    // This function will never be called with a terminal state as an argument.
    //
    // IMPORTANT: Although this trait definition provides a mutable reference to give maximum flexibility
    // to implementations, the invocation of the function should be referentially transparent.
    // (The same State should result in the same Hypotheses every time.)
    fn hypotheses(&mut self, state: &State) -> Hypotheses<Action>;

    // IMPORTANT: The same caveat about referential transparency applies to this function.
    fn next(&mut self, &State, &Action) -> State; // When MCTS choses an action for the first time, it will call this function to obtain the new state. Used during the MCTS leaf expansion step.

    // The results of search are returned as a vector of probabilities over the space of possible actions.
    // The action_space tells the cardinality of the (finite) set of possible actions for the game. E.g., in 19x19 Go, there are 363 possible actions -
    // 361 possible stone placements, pass, and resign. In a simple model of tic tac toe, there are 9.
    // The vector of search results will be dense vector of this size, with one entry per action, including
    fn max_actions(&mut self) -> usize;
}

type NodeIdx = petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>;
type EdgeIdx = petgraph::graph::EdgeIndex<petgraph::graph::DefaultIx>;

// An opaque token which can be efficient 'applied' to search tree to advance the root of the tree to the next node
#[derive(Debug)]
pub struct ApplicationToken(NodeIdx);

#[derive(Debug, Clone)]
struct Edge<Action>
where
    Action: self::Action,
{
    action: Action,     // The Action that this edge represents.
    prior: f32,         // The effective prior, after scaling and Dirichlet noise.
    scaled_prior: f32, // P(s, a). The prior probability of choosing this node, derived from the expert guess.
    raw_prior: f32, // The raw prior we got from the net, before rescaling legal actions to 1. Used for debugging.
    visit_count: usize, // # games played from this position
    wins: usize,        // Wins from this position
    losses: usize,      // Losses from this position
    // average_value: f64, // Q(s, a) in the AGZ paper. The average value of an action over all the times it's been tried. Equal to total_value / visit_count.
}

#[derive(Debug)]
struct Node<State>
where
    State: ::std::fmt::Debug,
{
    expanded: bool,
    state: State,
    visits: usize,
}
impl<State> Node<State>
where
    State: ::std::fmt::Debug,
{
    fn new_unexpanded(state: State) -> Self {
        Node {
            expanded: false,
            state: state,
            visits: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, StructOpt)]
#[structopt(
    name = "seraphim config",
    about = "Basic configuration that must be defined for any seraphim program."
)]
pub struct SeraphimConfig {
    #[structopt(env = "SERAPHIM_MODEL_NAME", long)]
    pub model_name: String,

    #[structopt(env = "SERAPHIM_DATA", long)]
    pub seraphim_data: String,
}

#[derive(Clone, Debug, PartialEq, StructOpt)]
#[structopt(
    name = "search options",
    about = "Hyperparamters for configuring tree search."
)]
pub struct SearchTreeParamOverrides {
    #[structopt(
        long,
        help = "A constant determining the tradeoff between exploration and exploitation. Higher values bias towards exploration."
    )]
    pub cpuct: Option<f32>,
    #[structopt(
        long = "raw",
        help = "Play the game using raw scores from the expert, without searching. Often used with --readouts 0"
    )]
    pub use_raw_scores: Option<bool>,

    #[structopt(
        long,
        help = "How many games to sample while searching for the next move."
    )]
    pub readouts: Option<u32>,

    #[structopt(
        long,
        help = "Move selection is tempered to always chose the highest probability move after this ply of the game."
    )]
    pub tempering_point: Option<u32>,

    #[structopt(long, help = "Noise coefficient for Dirichlet noise.")]
    pub noise_coefficient: Option<f32>,

    #[structopt(
        long,
        help = "This is a in Dir(a). Set this value proportional to the number of available actions for an average move of the game. #actions * dirichet_alpha ~= 10"
    )]
    pub dirichlet_alpha: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchTreeOptions {
    pub cpuct: f32,
    pub use_raw_scores: bool,
    pub readouts: u32,
    pub tempering_point: u32,
    pub noise_coefficient: f32,
    pub dirichlet_alpha: f64,
}
impl std::default::Default for SearchTreeOptions {
    // These defaults are the values used in the AGZ paper.
    fn default() -> Self {
        Self {
            use_raw_scores: false,
            cpuct: 0.25,
            readouts: 800,
            tempering_point: 30,
            dirichlet_alpha: 0.03,
            noise_coefficient: 0.25,
        }
    }
}
impl SearchTreeOptions {
    pub fn from_overrides(overrides: SearchTreeParamOverrides) -> Self {
        let default = Self::default();
        Self {
            use_raw_scores: overrides.use_raw_scores.unwrap_or(default.use_raw_scores),
            cpuct: overrides.cpuct.unwrap_or(default.cpuct),
            readouts: overrides.readouts.unwrap_or(default.readouts),
            tempering_point: overrides.tempering_point.unwrap_or(default.tempering_point),
            dirichlet_alpha: overrides.dirichlet_alpha.unwrap_or(default.dirichlet_alpha),
            noise_coefficient: overrides
                .noise_coefficient
                .unwrap_or(default.noise_coefficient),
        }
    }
}

#[derive(Debug)]
pub struct SearchTree<State, Action>
where
    State: self::State,
    Action: self::Action,
{
    rand: rand::rngs::ThreadRng,
    search_tree: petgraph::stable_graph::StableGraph<Node<State>, Edge<Action>>,
    ply: u32,          // how many plys have been played at the root_idx
    root_idx: NodeIdx, // cur
    options: SearchTreeOptions,
}

impl<State, Action> SearchTree<State, Action>
where
    State: self::State,
    Action: self::Action,
{
    // Start a new game that will be played by iterative searching
    pub fn init_with_options(initial_state: State, options: SearchTreeOptions) -> Self {
        let mut search_tree = petgraph::stable_graph::StableGraph::new();
        let root_node = Node::new_unexpanded(initial_state);
        let root_idx = search_tree.add_node(root_node);

        Self {
            search_tree,
            ply: 0,
            options,
            root_idx,
            rand: rand::thread_rng(),
        }
    }
    pub fn init(initial_state: State) -> Self {
        Self::init_with_options(initial_state, SearchTreeOptions::default())
    }

    pub fn current_state_ref(&self) -> &State {
        &self.search_tree[self.root_idx].state
    }

    // Read the next move and return the result without applying.
    // Note: This call will typically be followed by apply() or apply_search_results()
    pub fn read_debug(
        &mut self,
        game_expert: &mut GameExpert<State, Action>,
    ) -> SearchResultsDebugInfo<Action> {
        let now = time::Instant::now();
        let child_edges_pre_read: Vec<Edge<Action>> = self
            .search_tree
            .neighbors(self.root_idx)
            .map(|child_node_idx| {
                self.search_tree[self.parent_edge_idx(child_node_idx).unwrap()].clone()
            })
            .collect();

        let mut pre_read_edge_map = HashMap::new();
        for edge in child_edges_pre_read {
            pre_read_edge_map.insert(edge.action.clone(), edge.clone());
        }

        self.readout(game_expert);

        let child_edges: Vec<Edge<Action>> = self
            .search_tree
            .neighbors(self.root_idx)
            .map(|child_node_idx| {
                self.search_tree[self.parent_edge_idx(child_node_idx).unwrap()].clone()
            })
            .collect();

        let total_visit_count = self.search_tree[self.root_idx].visits;

        let mut candidates: Vec<CandidateActionDebugInformation<Action>> =
            Vec::with_capacity(child_edges.len());
        for edge in child_edges.iter().rev() {
            // reversing because the regular graph enumerate is LIFO and we want FIFO
            let pre_read_visit_count = pre_read_edge_map
                .get(&edge.action)
                .map(|e| e.visit_count)
                .unwrap_or(0);
            let stimulus = self.exploration_stimulus(&edge, total_visit_count);
            
            let n = edge.visit_count;
            let w = edge.wins as i64 - edge.losses as i64;

            let q = w as f64 / n as f64;

            candidates.push(CandidateActionDebugInformation {
                action: edge.action.clone(),
                prior: edge.prior,
                raw_prior: edge.raw_prior,
                scaled_prior: edge.scaled_prior,
                posterior: (edge.visit_count as f32) / (total_visit_count as f32),
                wins: edge.wins,
                losses: edge.losses,
                total_visits: edge.visit_count,
                visits_in_last_read: edge.visit_count - pre_read_visit_count,
                average_value: q,
                exploration_stimulus: stimulus,
            });
        }
        let hot = self.ply < self.options.tempering_point;
        let results = self.select(game_expert);
        let elapsed = now.elapsed();
        SearchResultsDebugInfo {
            time: elapsed,
            results,
            hot,
            candidates,
        }
    }

    pub fn read(
        &mut self,
        game_expert: &mut GameExpert<State, Action>,
    ) -> SearchResultsInfo<Action> {
        self.readout(game_expert);
        self.select(game_expert)
    }
    pub fn apply_search_results(&mut self, result: &SearchResultsInfo<Action>) {
        self.advance_to_node(result.application_token.0);
    }

    // update the search tree by applying an action.
    pub fn apply(&mut self, action: &Action) {
        let next_node_idx = self
            .search_tree
            .neighbors(self.root_idx)
            .find(|node_idx| {
                &self.search_tree[self.parent_edge_idx(*node_idx).unwrap()].action == action
            })
            .unwrap();
        self.advance_to_node(next_node_idx);
    }

    fn readout(&mut self, game_expert: &mut GameExpert<State, Action>) {
        for _ in 0..self.options.readouts {
            self.read_to_end(game_expert);
        }
    }
    // After sampling is done, it's time to select the next action that will actually be played.
    // The AGZ algorithm uses tempering. Before tempering, it selects the next actions in proportion to the number of times each action was sampled.
    // After tempering, it choses the next action with the highest number of samples.
    fn select(&mut self, game_expert: &mut GameExpert<State, Action>) -> SearchResultsInfo<Action> {
        /*
        Select a next move using the AGZ annealing method:

            At the end of the search AGZ selects a move to play
            in the root position proportional to its exponentiated visit count
            p(a|s_0) = N(s_0, a)^{1/T} / \sum{b}N(s_0, b)^{1/T},
            where T is a temperature parameter. The search tree is reused at subsequent
            timesteps: the played action becomes the new root node, and other nodes are discarded.

            AGZ resigns if its root value and best child value are lower than a threshhold, but Seraphim won't, yet.

        Apply that selection to the search tree by advancing down the tree, and return the picked action so that the GameExpert will know what was picked.

        The GameExpert should train itself to be more likely to pick the same value that the tree search picked. SearchResultsInfo
        gives the label that should be trained.

        TODO: Include win probability as a feature.
        */

        let max_actions = game_expert.max_actions();
        let self_ptr = self as *mut Self;
        let mut guard = 0;
        loop {
            // There could be a fp error so that total probably is slightly less than 1. This almost never occurs, but if it does, we'll just resample.
            if self.options.use_raw_scores {
                self.expand(self.root_idx, game_expert);
            }
            let child_edges: Vec<(&Edge<Action>, NodeIdx)> = self
                .search_tree
                .neighbors(self.root_idx)
                .map(|child_node_idx| {
                    (
                        &self.search_tree[self.parent_edge_idx(child_node_idx).unwrap()],
                        child_node_idx,
                    )
                })
                .collect();
            let mut results = vec![0.0; max_actions];

            let total_visit_count = self.search_tree[self.root_idx].visits;

            if self.options.use_raw_scores {
                for (edge, _) in &child_edges {
                    results[edge.action.index()] = edge.raw_prior;
                }
            } else {
                for (edge, _) in &child_edges {
                    let prob: f32 = (edge.visit_count as f32) / (total_visit_count as f32);
                    results[edge.action.index()] = prob;
                }
            }
            if self.ply < self.options.tempering_point {
                for (i, (edge, node_idx)) in child_edges.iter().enumerate() {
                    let rand: f32 = unsafe { (*self_ptr).rand.gen_range(0.0, 1.0) }; // Ok, because we don't rely any other state in for random number gen
                    let mut cum_prob = 0.0;

                    cum_prob += results[edge.action.index()];
                    if cum_prob > rand {
                        let selection = edge.action.clone();
                        return SearchResultsInfo {
                            results,
                            selection,
                            application_token: ApplicationToken(*node_idx),
                        };
                    }
                }
            } else {
                let (selected_edge, selected_node) = if self.options.use_raw_scores {
                    child_edges
                        .iter()
                        .max_by(|(a, _), (b, _)| {
                            a.raw_prior
                                .partial_cmp(&b.raw_prior)
                                .unwrap_or(std::cmp::Ordering::Less)
                        })
                        .unwrap()
                } else {
                    child_edges
                        .iter()
                        .max_by_key(|(e, _)| e.visit_count)
                        .unwrap()
                };

                let selection = selected_edge.action.clone();
                return SearchResultsInfo {
                    results,
                    selection,
                    application_token: ApplicationToken(*selected_node),
                };
            }
            if guard > 5 {
                panic!("infinite loop");
            }
        }
    }

    fn advance_to_node(&mut self, node: NodeIdx) {
        self.ply += 1;
        self.root_idx = node; // TODO: During a normal game, we can save memory by forgetting all nodes that are above the analysis root: those paths will not be further explored.
    }

    /*
        The next node to sample is the node that maximizes
        exploration_stimulus = Q(s, a) + U(s, a)

        where

        U(s, a) = cP(s,a)sqrt(Nb)/(1 + Na)

        Q(s, a) is the average reward for exploring that node in the past. It is equal to wins/nb

        P is the prior probability that the action is the best
        Na is the number of visits of to this edge,
        Nb is the number of visits to the parent edge,
        c is "a constant determining the level of exploration".
    */
    
    fn exploration_stimulus(&self, edge: &Edge<Action>, parent_visits: usize) -> f64 {
        let N = parent_visits as f64; // how many times the parent state has been visited
        let n = edge.visit_count as f64 + 1.0f64; // how many times this action has been explored from the parent state

        let p = edge.prior as f64; // prior probability that this action is the best available
        let w = edge.wins as i64 - edge.losses as i64;
        let q = w as f64 / n as f64; 
        
        let c = self.options.cpuct as f64;
        let u = c * p * (N.ln() / n).sqrt();

        return q + u;
    }
    
    // We always sample the node that has the highest exploration_stimulus, as described above
    fn next_node_to_sample(
        &self,
        idx: NodeIdx,
        game_expert: &mut GameExpert<State, Action>,
    ) -> NodeIdx {

        let parent_visits: usize = self.search_tree[idx].visits;
            
        let explorations = self
            .search_tree
            .neighbors(idx)
            .map(|node_idx| {
                let edge_idx = self.parent_edge_idx(node_idx).unwrap();
                let edge = &self.search_tree[edge_idx];
                let exploration_stimulus = self.exploration_stimulus(&edge, parent_visits);
                (edge_idx, exploration_stimulus)
            });

        let (idx_with_max_stimulus, _) = explorations.max_by(
                |&(_, exploration_stimulus_a), &(_, exploration_stimulus_b)| {
                    exploration_stimulus_a
                        .partial_cmp(&exploration_stimulus_b)
                        .unwrap_or(::std::cmp::Ordering::Equal)
                },
            )
            .unwrap();

        let (_, next_node_idx) = self.search_tree.edge_endpoints(idx_with_max_stimulus).unwrap();
        next_node_idx
    }

    fn parent_node_idx(&self, idx: NodeIdx) -> Option<NodeIdx> {
        self.search_tree
            .neighbors_directed(idx, petgraph::Direction::Incoming)
            .nth(0)
    }

    fn parent_edge_idx(&self, idx: NodeIdx) -> Option<EdgeIdx> {
        self.parent(idx).map(|(_, e)| e)
    }

    fn parent(&self, idx: NodeIdx) -> Option<(NodeIdx, EdgeIdx)> {
        let parent_node = self.parent_node_idx(idx);
        if !parent_node.is_some() {
            return None;
        }
        let parent_edge = parent_node.and_then(|parent_idx| self.search_tree.find_edge(parent_idx, idx)).unwrap();
        Some((parent_node.unwrap(), parent_edge))
    }

    // follow the search to a terminal node (A node where GameStatus is not InProgress),
    // then back up the tree to the current analysis root (e.g., the state of the game as it has played out thus far),
    // updating win, loss, and visit counts.
    fn read_to_end(&mut self, game_expert: &mut GameExpert<State, Action>) {
        let self_ptr = self as *mut Self;
        unsafe {
            let mut node_idx = (*self_ptr).root_idx;
            let mut node_weight = &(*self_ptr).search_tree[node_idx];

            while node_weight.state.status() == GameStatus::InProgress {
                if !node_weight.expanded {
                    self.expand(node_idx, game_expert);
                }
                node_idx = self.next_node_to_sample(node_idx, game_expert);
                node_weight = &(*self_ptr).search_tree[node_idx];
            }
            self.backup(node_idx);
        }
    }
    // record the result of a single readout up to the analysis root, which during normal operation
    // is the node representating the current state of the game as it has evolved so far. In other words,
    // it is the node we're starting our search from.
    fn backup(&mut self, node_idx: NodeIdx) {
        let node_weight = &self.search_tree[node_idx];
        let status = node_weight.state.status();
        let mut win = match status {
            GameStatus::LastPlayerWon => 1,
            _ => 0,
        };
        let mut loss = match status {
            GameStatus::LastPlayerLost => 1,
            _ => 0
        };
        let mut node_idx = node_idx;
        loop {
            let parent = self.parent(node_idx);
            if parent.is_none() {
                break;
            }
            let (parent_node_idx, parent_edge_idx) = parent.unwrap();
            if parent_node_idx == self.root_idx {
                break;
            }
            let parent_edge_weight = self.search_tree.edge_weight_mut(parent_edge_idx).unwrap();

            parent_edge_weight.wins += win;
            parent_edge_weight.losses += loss;
            parent_edge_weight.visit_count += 1;

            std::mem::swap(&mut win, &mut loss);

            node_idx = parent_node_idx;
        }
    }
    // For each possible action from the current state (node_idx), create a new node and edge representing the application of that action.
    fn expand(&mut self, node_idx: NodeIdx, game_expert: &mut GameExpert<State, Action>) {
        {
            let node = &self.search_tree[node_idx];
            let state = &node.state;

            match &state.status() {
                &GameStatus::InProgress => {}
                _ => {
                    return;
                }
            }
        }
        {
            let self_ptr = self as *mut Self;
            let Hypotheses {
                mut legal_actions, ..
            } = game_expert.hypotheses(&self.search_tree[node_idx].state);

            let mut total_probability = 0.0;
            for (_, p) in &legal_actions {
                total_probability += p;
            }
            let c = self.options.noise_coefficient;
            let scale = 1.0 / total_probability; // Ok to divide by because at least one action must have non-zero probability
            let effective_priors: Vec<f32> = if legal_actions.len() > 1 {
                let dist =
                    Dirichlet::new_with_param(self.options.dirichlet_alpha, legal_actions.len());
                let sample = dist.sample(&mut rand::thread_rng());

                legal_actions
                    .iter()
                    .zip(sample)
                    .map(|((_, p), n)| ((*p) * scale * (1.0 - c)) + (n as f32 * c))
                    .collect()
            } else {
                legal_actions.iter().map(|(_, p)| *p).collect()
            };

            for (i, (action, raw_prior)) in legal_actions.into_iter().enumerate() {
                let new_state = game_expert.next(&self.search_tree[node_idx].state, &action);
                let e = self.options.noise_coefficient;

                unsafe {
                    let leaf_idx = (*self_ptr)
                        .search_tree
                        .add_node(Node::new_unexpanded(new_state));

                    (*self_ptr).search_tree.add_edge(
                        node_idx,
                        leaf_idx,
                        Edge {
                            action,
                            prior: effective_priors[i],
                            scaled_prior: raw_prior * scale,
                            raw_prior,
                            visit_count: 0,
                            wins: 0,
                            losses: 0,
                        },
                    );
                }
            }
        }
        self.search_tree.node_weight_mut(node_idx).unwrap().expanded = true;
    }
}
