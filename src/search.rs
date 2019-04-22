//! This module implements informed Monte Carlo Tree Search. "Informed" means that the search is guided by an expert policy
//! that ascribes Bayesian prior probabilities to question of whether each possible next action is the best one.
//! Consumers of the Seraphim library are to implement the GameExpert trait, and pass an instance of GameExpert
//! to SearchTree.
use petgraph;
use rand::distributions::{Dirichlet, Distribution, Uniform};

use crate::{
    game,
    game::GameStatus,
    inference
};
use std::{collections::HashMap, default::Default, time};

use crate::error::Result;

#[derive(Debug)]
pub struct CandidateActionDebugInformation {
    pub action: usize,
    pub prior: f32,     // Prior after scaling and noise
    pub posterior: f32, // The improved probability that this move is the best after PUCT search
    pub raw: f32,
    pub total_visits: usize, // how many times has this line of play been sampled, in total
    pub wins: usize,
    pub losses: usize,
    pub visits_in_last_read: usize, // how many times was this line of play sampled in the most recent read
    pub average_value: f64,         // The average value of taking this action Q(s, a) in the paper
    pub exploration_stimulus: f64, // How badly does the search tree want to explore this action in the future?
                                   // The highest value of here is the node it would sample next if asked to perform more readouts.
}

#[derive(Debug)]
pub struct SearchResultsDebugInfo {
    pub time: time::Duration, // How long it took to compute this move
    pub candidates: Vec<CandidateActionDebugInformation>,
    pub results: SearchResultsInfo,
    pub hot: bool, // Was this move chosen from a cold or hot sample. Hot introduces noise early in the game to ensure game variety.
}

#[derive(Debug)]
pub struct SearchResultsInfo {
    pub results: Vec<f32>,
    pub selection: usize,
    pub application_token: ApplicationToken,
}

type NodeIdx = petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>;
type EdgeIdx = petgraph::graph::EdgeIndex<petgraph::graph::DefaultIx>;

// An opaque token which can be efficient 'applied' to search tree to advance the root of the tree to the next node
#[derive(Debug)]
pub struct ApplicationToken(NodeIdx);

#[derive(Debug)]
struct PValues {
    // The raw prior we got from the net, before normalizing legal actions to sum to 1 and
    // introducing noise
    raw_priors: crate::inference::Priors,
    // P(s, a). The prior probability of choosing this node, derived from the expert guess.
    noised_and_scaled_priors: Vec<f32>,
}

#[derive(Debug, Clone)]
struct Edge
where
{
    action: usize, // The action that this edge represents.
    prior: f32,    // The effective prior, after scaling and Dirichlet noise.
    raw_prior: f32,
    visit_count: usize, // # games played from this position
    wins: usize,        // Wins from this position
    losses: usize,      // Losses from this position
                        // average_value: f64, // Q(s, a) in the AGZ paper. The average value of an action over all the times it's been tried. Equal to total_value / visit_count.
}

#[derive(Debug)]
struct Node<State>
where
    State: crate::game::GameState,
{
    expanded: bool,
    state: State,
    visits: usize,
}
impl<State> Node<State>
where
    State: crate::game::GameState,
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

    #[structopt(
        long,
        help = "Noise coefficient for Dirichlet noise (what fraction of probability is from Dir(a))."
    )]
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
pub struct SearchTree<Inference, State, Game>
where
    Inference: inference::Inference,
    State: game::GameState,
    Game: game::Game<State = State>,
{
    inference: Inference,
    rand: rand::rngs::ThreadRng,
    uniform: Uniform<f32>,
    search_tree: petgraph::stable_graph::StableGraph<Node<State>, Edge>,
    ply: u32,
    root_idx: NodeIdx,
    options: SearchTreeOptions,
    game: Game,
}
impl<Inference, State, Game> SearchTree<Inference, State, Game>
where
    Inference: inference::Inference,
    State: game::GameState,
    Game: game::Game<State = State>,
{
    // Start a new game that will be played by iterative searching
    pub fn init_with_options(inference: Inference, game: Game, options: SearchTreeOptions) -> Self {
        let mut search_tree = petgraph::stable_graph::StableGraph::new();
        let root_node = Node::new_unexpanded(State::default());
        let root_idx = search_tree.add_node(root_node);

        Self {
            inference,
            search_tree,
            ply: 0,
            options,
            root_idx,
            rand: rand::thread_rng(),
            uniform: Uniform::<f32>::from(0.0..1.0),
            game,
        }
    }
    pub fn init(inference: Inference, game: Game) -> Self {
        Self::init_with_options(inference, game, SearchTreeOptions::default())
    }

    pub fn game_ref(&self) -> &Game {
        &self.game
    }

    pub fn current_state_ref(&self) -> &State {
        &self.search_tree[self.root_idx].state
    }

    pub fn status(&self) -> GameStatus {
        self.game.status(self.current_state_ref())
    }

    pub fn action_count(&self) -> usize {
        self.game.action_count()
    }


    // Read the next move and return the result without applying.
    // Note: This call will typically be followed by apply() or apply_search_results()
    pub fn read_debug(&mut self) -> Result<SearchResultsDebugInfo> {
        let now = time::Instant::now();
        let child_edges_pre_read: Vec<Edge> = self
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

        self.readout()?;

        let child_edges: Vec<Edge> = self
            .search_tree
            .neighbors(self.root_idx)
            .map(|child_node_idx| {
                self.search_tree[self.parent_edge_idx(child_node_idx).unwrap()].clone()
            })
            .collect();

        let total_visit_count = self.search_tree[self.root_idx].visits;

        let mut candidates: Vec<CandidateActionDebugInformation> =
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
                raw: edge.raw_prior,
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
        let results = self.select();
        let elapsed = now.elapsed();
        Ok(SearchResultsDebugInfo {
            time: elapsed,
            results,
            hot,
            candidates,
        })
    }

    pub fn read(&mut self) -> Result<SearchResultsInfo> {
        self.readout()?;
        Ok(self.select())
    }
    pub fn apply_search_results(&mut self, result: &SearchResultsInfo) {
        self.advance_to_node(result.application_token.0);
    }

    // update the search tree by applying an action.
    pub fn apply(&mut self, action: usize) {
        let next_node_idx = self
            .search_tree
            .neighbors(self.root_idx)
            .find(|node_idx| {
                self.search_tree[self.parent_edge_idx(*node_idx).unwrap()].action == action
            })
            .unwrap();
        self.advance_to_node(next_node_idx);
    }

    fn readout(&mut self) -> Result<()> {
        for _ in 0..self.options.readouts {
            self.read_to_end()?;
        }
        Ok(())
    }
    // After sampling is done, it's time to select the next action that will actually be played.
    // The AGZ algorithm uses tempering. Before tempering, it selects the next actions in proportion to the number of times each action was sampled.
    // After tempering, it choses the next action with the highest number of samples.
    fn select(&mut self) -> SearchResultsInfo {
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

        let max_actions = self.game.action_count();
        let self_ptr = self as *mut Self;
        let mut guard = 0;

        loop {
            guard += 1;

            // There could be a fp error so that total probably is slightly less than 1. This almost never occurs, but if it does, we'll just resample.
            let child_edges: Vec<(&Edge, NodeIdx)> = self
                .search_tree
                .neighbors(self.root_idx)
                .map(|child_node_idx| {
                    (
                        &self.search_tree[self.parent_edge_idx(child_node_idx).unwrap()],
                        child_node_idx,
                    )
                })
                .collect();

            if cfg!(debug_assertions) {
                trace!("Edges during selection:");
                for edge in &child_edges {
                    trace!("{:?}", edge);
                }
            }
            let mut results = vec![0.0; max_actions];

            let total_visit_count = self.search_tree[self.root_idx].visits;

            if self.options.use_raw_scores {
                for (edge, _) in &child_edges {
                    results[edge.action] = edge.raw_prior;
                }
            } else {
                for (edge, _) in &child_edges {
                    let prob: f32 = (edge.visit_count as f32) / (total_visit_count as f32);
                    trace!("{} / {}", edge.visit_count, total_visit_count);
                    results[edge.action] = prob;
                }
            }
            if self.ply < self.options.tempering_point {
                let mut cum_prob = 0.0;
                let rand: f32 = unsafe { self.uniform.sample(&mut (*self_ptr).rand) }; // Ok, because we don't rely any other state in for random number gen
                for (_i, (edge, node_idx)) in child_edges.iter().enumerate() {
                    cum_prob += results[edge.action];
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
            if cfg!(debug_assertions) {
                if guard > 2 {
                    let foo = 10 * 12;
                    println!("{}", foo);
                }
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

    fn exploration_stimulus(&self, edge: &Edge, parent_visits: usize) -> f64 {
        let N = parent_visits as f64; // how many times the parent state has been visited
        let n = edge.visit_count as f64 + 1.0f64; // how many times this action has been explored from the parent state

        let p = edge.prior as f64; // prior probability that this action is the best available
        let w = edge.wins as i64 - edge.losses as i64;
        let q = w as f64 / n as f64;

        let c = self.options.cpuct as f64;
        let u = c * p * (N / n).sqrt();
        if cfg!(debug_assertions) {
            if (q + u).is_nan() {
                warn!("Exploration is NAN! at {:?}", edge);
            }
        }
        return q + u;
    }

    // We always sample the node that has the highest exploration_stimulus, as described above
    // Should be next_action_to_sample(?!)
    fn next_node_to_sample(&self, idx: NodeIdx) -> NodeIdx {
        let parent_visits: usize = self.search_tree[idx].visits;

        let explorations = self.search_tree.neighbors(idx).map(|node_idx| {
            let edge_idx = self.parent_edge_idx(node_idx).unwrap();
            let edge = &self.search_tree[edge_idx];
            let exploration_stimulus = self.exploration_stimulus(&edge, parent_visits);
            (edge_idx, exploration_stimulus)
        });

        if cfg!(debug_assertions) {
            let explorations = self.search_tree.neighbors(idx).map(|child_idx| {
                let edge_idx = self.parent_edge_idx(child_idx).unwrap();
                let edge = &self.search_tree[edge_idx];
                let stimulus = self.exploration_stimulus(&edge, parent_visits);
                (edge_idx, edge, stimulus)
            });
            for (e_idx, e, s) in explorations {
                let (_, node_idx) = self.search_tree.edge_endpoints(e_idx).unwrap();
                trace!("{:?}\n{:?}: {:?}", e, node_idx, s);
            }
        }

        let (idx_with_max_stimulus, _) = explorations
            .into_iter()
            .max_by(
                |&(_, exploration_stimulus_a), &(_, exploration_stimulus_b)| {
                    exploration_stimulus_a
                        .partial_cmp(&exploration_stimulus_b)
                        .unwrap_or(::std::cmp::Ordering::Equal)
                },
            )
            .unwrap();

        let (_, next_node_idx) = self
            .search_tree
            .edge_endpoints(idx_with_max_stimulus)
            .unwrap();
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
        let parent_edge = parent_node
            .and_then(|parent_idx| self.search_tree.find_edge(parent_idx, idx))
            .unwrap();
        Some((parent_node.unwrap(), parent_edge))
    }

    // follow the search to a terminal node (A node where GameStatus is not InProgress),
    // then back up the tree to the current analysis root (e.g., the state of the game as it has played out thus far),
    // updating win, loss, and visit counts.
    fn read_to_end(&mut self) -> Result<()> {
        let mut node_idx = self.root_idx;
        let mut node_weight = &mut self.search_tree[node_idx];

        while self.game.status(&node_weight.state) == GameStatus::InProgress {
            node_weight.visits += 1;
            if !node_weight.expanded {
                self.expand(node_idx)?;
            }
            node_idx = self.next_node_to_sample(node_idx);
            node_weight = &mut self.search_tree[node_idx];
        }
        self.backup(node_idx);
        Ok(())
    }
    // record the result of a single readout up to the analysis root, which during normal operation
    // is the node representating the current state of the game as it has evolved so far. In other words,
    // it is the node we're starting our search from.
    fn backup(&mut self, node_idx: NodeIdx) {
        let node_weight = &self.search_tree[node_idx];
        let status = self.game.status(&node_weight.state);
        let mut win = match status {
            GameStatus::LastPlayerWon => 1,
            _ => 0,
        };
        let mut loss = match status {
            GameStatus::LastPlayerLost => 1,
            _ => 0,
        };
        let mut node_idx = node_idx;
        loop {
            let parent = self.parent(node_idx);
            // if parent.is_none() {
            //     break;
            // }
            let (parent_node_idx, parent_edge_idx) = parent.unwrap();
            let parent_edge_weight = self.search_tree.edge_weight_mut(parent_edge_idx).unwrap();

            parent_edge_weight.wins += win;
            parent_edge_weight.losses += loss;
            parent_edge_weight.visit_count += 1;

            std::mem::swap(&mut win, &mut loss);

            node_idx = parent_node_idx;
            if node_idx == self.root_idx {
                break;
            }
        }
    }

    fn pvalues(&mut self, node_idx: NodeIdx) -> Result<PValues> {
        let state = &self.search_tree[node_idx].state;
        let legal_actions = self.game.legal_actions(&state);
        let state_bytes = state.feature_bytes().into_iter().collect::<Vec<u8>>();
        let raw_priors = self.inference.infer(&state_bytes[..])?;
        let sum: f32 = raw_priors.ps.iter().sum();

        let scale = 1.0 / sum;
        let c = self.options.noise_coefficient;

        let dist = Dirichlet::new_with_param(self.options.dirichlet_alpha, legal_actions.len());
        let sample = dist.sample(&mut rand::thread_rng());

        let noised_and_scaled_priors = raw_priors
            .ps
            .iter()
            .zip(legal_actions)
            .map(|(&p, l)| if l { p } else { 0.0 })
            .zip(sample)
            .map(|(p, n)| (p * scale * (1.0 - c)) + (n as f32 * c))
            .collect();

        Ok(PValues {
            raw_priors,
            noised_and_scaled_priors,
        })
    }

    fn expand(&mut self, node_idx: NodeIdx) -> Result<()> {
        let pvalues: PValues = self.pvalues(node_idx)?;

        for (i, p) in pvalues.noised_and_scaled_priors.iter().enumerate() {
            if *p == 0.0 {
                continue;
            }

            let new_state = self.game.successor(&self.search_tree[node_idx].state, i);

            let leaf_idx = self.search_tree.add_node(Node::new_unexpanded(new_state));

            self.search_tree.add_edge(
                node_idx,
                leaf_idx,
                Edge {
                    action: i,
                    raw_prior: pvalues.raw_priors.ps[i],
                    prior: pvalues.noised_and_scaled_priors[i],
                    visit_count: 0,
                    wins: 0,
                    losses: 0,
                },
            );
        }
        self.search_tree.node_weight_mut(node_idx).unwrap().expanded = true;
        Ok(())
    }
}
