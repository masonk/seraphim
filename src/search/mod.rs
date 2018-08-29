//! This module implements informed Monte Carlo tree search. "Informed" means that the search is guided by an expert policy
//! that ascribes Bayesian prior probabilities to question of whether each possible next action is the best one.
//! Consumers of the Seraphim library are to implement the GameExpert trait, and pass an instance of GameExpert
//! to SearchTree.
use petgraph;
use rand;
use rand::Rng;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameStatus {
    InProgress,
    TerminatedWithoutResult,
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
    pub prior: f32,               // The naive probability that this move is the best
    pub posterior: f32, // The improved probability that this move is the best after PUCT search
    pub total_visits: u32, // how many times has this line of play been sampled, in total
    pub visits_in_last_read: u32, // how many times was this line of play sampled in the most recent read
    pub average_value: f32,       // The average value of taking this action Q(s, a) in the paper
    pub total_value: f32,         // W(s, a) in the paper
    pub exploration_stimulus: f32, // How badly does the search tree want to explore this action in the future?
                                   // The highest value of here is the node it would sample next if asked to perform more readouts.
}

#[derive(Debug)]
pub struct SearchResultsDebugInfo<Action>
where
    Action: self::Action,
{
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
    prior: f32, // P(s, a). The prior probability of choosing this node, derived from the expert guess.
    visit_count: u32, // N(s, a) in the AGZ paper. How many times has the action that this edge represents been tried so far?
    total_value: f32, // W(s, a) in the AGZ paper. The total value of an action over all the times it's been tried.
    average_value: f32, // Q(s, a) in the AGZ paper. The average value of an action over all the times it's been tried. Equal to total_value / visit_count.
}

#[derive(Debug)]
struct Node<State>
where
    State: ::std::fmt::Debug,
{
    expanded: bool,
    state: State,
}
impl<State> Node<State>
where
    State: ::std::fmt::Debug,
{
    fn new_unexpanded(state: State) -> Self {
        Node {
            expanded: false,
            state: state,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchTreeOptions {
    pub cpuct: f32, // a constant determining the tradeoff between exploration and exploitation; .25 in the AGZ paper.
    // Higher numbers bias the search towards less-explored nodes, lower numbers bias the search
    // towards more promising nodes.
    pub readouts: u32, // How many games to play when searching for a single move; 1600 in the AGZ paper.
    pub tempering_point: u32, // how many plys should progress until we lower the temperature of move selection from 1
                              // to ~0. 30 in the AGZ paper
}
impl SearchTreeOptions {
    // These defaults are the values used in the AGZ paper.
    pub fn defaults() -> Self {
        Self {
            cpuct: 0.25,
            readouts: 1600,
            tempering_point: 30,
        }
    }
}
#[derive(Debug)]
pub struct SearchTree<State, Action>
where
    State: self::State,
    Action: self::Action,
{
    rand: rand::ThreadRng,
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
        Self::init_with_options(initial_state, SearchTreeOptions::defaults())
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
        let child_edges_pre_read: Vec<Edge<Action>> = self.search_tree
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

        let child_edges: Vec<Edge<Action>> = self.search_tree
            .neighbors(self.root_idx)
            .map(|child_node_idx| {
                self.search_tree[self.parent_edge_idx(child_node_idx).unwrap()].clone()
            })
            .collect();

        let total_visit_count: u32 = child_edges.iter().map(|e| e.visit_count).sum();

        let mut candidates: Vec<CandidateActionDebugInformation<Action>> =
            Vec::with_capacity(child_edges.len());
        for edge in child_edges.iter().rev() {
            // reversing because the regular graph enumerate is LIFO and we want FIFO
            let pre_read_visit_count = pre_read_edge_map
                .get(&edge.action)
                .map(|e| e.visit_count)
                .unwrap_or(0);
            let stimulus = self.exploration_stimulus(&edge, total_visit_count);
            candidates.push(CandidateActionDebugInformation {
                action: edge.action.clone(),
                prior: edge.prior,
                posterior: (edge.visit_count as f32) / (total_visit_count as f32),
                total_visits: edge.visit_count,
                visits_in_last_read: edge.visit_count - pre_read_visit_count,
                average_value: edge.average_value,
                total_value: edge.total_value,
                exploration_stimulus: stimulus,
            });
        }
        let hot = self.ply < self.options.tempering_point;
        let results = self.select(game_expert);
        SearchResultsDebugInfo {
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
        let next_node_idx = self.search_tree
            .neighbors(self.root_idx)
            .find(|node_idx| {
                &self.search_tree[self.parent_edge_idx(*node_idx).unwrap()].action == action
            })
            .unwrap();
        self.advance_to_node(next_node_idx);
    }

    fn readout(&mut self, game_expert: &mut GameExpert<State, Action>) {
        for _ in 0..self.options.readouts {
            self.read_to_end(self.root_idx, game_expert);
        }
    }
    // After visitation is done, it's time to select the next action that will actually be played.
    // The AGZ algorithm uses tempering. Before tempering, it selects the next action with probability proportional to visit counts.
    // After tempering, it just choses the next action with the highest count.
    fn select(&mut self, game_expert: &mut GameExpert<State, Action>) -> SearchResultsInfo<Action> {
        /*
        Read the board by playing out a number of games according to the PUCB MCTS algorithm.

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
        let self_ptr = self as *mut Self;
        let max_actions = game_expert.max_actions();
        loop {
            // There could be a fp error so that total probably is slightly less than 1. This almost never occurs, but if it does, we'll just resample.

            let child_edges: Vec<(&Edge<Action>, NodeIdx)> = self.search_tree
                .neighbors(self.root_idx)
                .map(|child_node_idx| {
                    (
                        &self.search_tree[self.parent_edge_idx(child_node_idx).unwrap()],
                        child_node_idx,
                    )
                })
                .collect();
            let mut results = vec![ 0.0; max_actions ];

            let total_visit_count: u32 = self.search_tree
                .neighbors(self.root_idx)
                .map(|n| (self.parent_edge_idx(n).unwrap(), n))
                .map(|(e, _)| {
                    let edge = self.search_tree.edge_weight(e).unwrap();
                    edge.visit_count
                })
                .sum();

            for (edge, _) in &child_edges {
                let prob: f32 = (edge.visit_count as f32) / (total_visit_count as f32);
                results[edge.action.index()]= prob;
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
                let (selected_edge, selected_node) = child_edges
                    .iter()
                    .max_by_key(|(e, _)| e.visit_count)
                    .unwrap();

                let selection = selected_edge.action.clone();
                return SearchResultsInfo {
                    results,
                    selection,
                    application_token: ApplicationToken(*selected_node),
                };
            }
        }
    }

    fn advance_to_node(&mut self, node: NodeIdx) {
        self.ply += 1;
        self.root_idx = node; // TODO: Remove all nodes that are made unreachable due to advancing down the tree.
    }

    // recursively follow the search to a terminal node (A node where GameStatus is not InProgress),
    // then back up the tree, updating edge weights.f
    fn read_to_end(&mut self, node_idx: NodeIdx, game_expert: &mut GameExpert<State, Action>) {
        let self_ptr = self as *mut Self;
        unsafe {
            let node = (*self_ptr).search_tree.node_weight_mut(node_idx).unwrap();
            let status = node.state.status();
            match status {
                GameStatus::InProgress => {
                    if !node.expanded {
                        self.expand(node_idx, game_expert);
                    }
                    let next_idx = self.select_next_node(node_idx, game_expert);

                    return self.read_to_end(next_idx, game_expert);
                }
                _ => {
                    match status {
                        GameStatus::LastPlayerLost => self.backup(0.0, node_idx),
                        GameStatus::LastPlayerWon => self.backup(1.0, node_idx),
                        GameStatus::TerminatedWithoutResult => self.backup(0.5, node_idx),
                        _ => {
                            warn!("Tried to backup some unknown terminal state. Algorithm error. ")
                        }
                    };
                }
            }
        }
    }

    /* 
        The next node to sample is the node that maximizes
        exploration_stimulus = Q(s, a) + U(s, a)

        where

        U(s, a) = cP(s,a)sqrt(Nb)/(1 + Na)

        Q(s, a) is the average reward for exploring that node in the past. It is equal to wins/nb 

        P is the prior probability that the action   is the best
        Na is the number of visits of to this edge,
        Nb is the number of visits to the parent edge,
        c is "a constant determining the level of exploration".
    */
    fn exploration_stimulus(&self, edge: &Edge<Action>, parent_visits: u32) -> f32 {
        self.options.cpuct * edge.prior * f32::sqrt(parent_visits as f32)
            / ((1 + edge.visit_count) as f32) + edge.average_value
    }
    fn select_next_node(
        &self,
        idx: NodeIdx,
        game_expert: &mut GameExpert<State, Action>,
    ) -> NodeIdx {
        let n_b: u32 = self.search_tree
            .neighbors(idx)
            .map(|child_idx| self.parent_edge_idx(child_idx).unwrap())
            .map(|edge_idx| {
                if let Some(edge) = self.search_tree.edge_weight(edge_idx) {
                    return edge.visit_count;
                }
                debug!(
                    "PROBLEM: NO PARENT FOR CHILD OF:\n{}",
                    self.search_tree[idx].state
                );
                panic!();
            })
            .sum();

        if cfg!(debug_assertions) {
            if n_b == 1000 {
                trace!("+++++++++ Debugging the choice for randomly chosen single move of a single readout ++++++++++++++");
                let parent = &self.search_tree[idx];
                trace!("For state\n{}:", parent.state);

                let pucts = self.search_tree.neighbors(idx).map(|node_idx| {
                    let edge_idx = self.parent_edge_idx(node_idx).unwrap();

                    let edge = &self.search_tree[edge_idx];
                    let u = self.options.cpuct * edge.prior * f32::sqrt(n_b as f32)
                        / ((1 + edge.visit_count) as f32);
                    let q = edge.average_value;
                    let uq = u + q;
                    (
                        edge,
                        u,
                        q,
                        uq,
                        game_expert.next(&parent.state, &edge.action),
                    )
                });

                for (e, u, q, uq, s) in pucts {
                    trace!("----- {:?} -----\n{}\nu: {:.3}\nq: {:.3}\nu+q: {:.3}\np: {:.3}\nq: {:.3}\nN: {}\n----\n", 
                    e.action,
                    s,
                    u, q, uq,
                    e.prior,
                    e.average_value,
                    e.visit_count);
                }

                trace!("+++++++++++++++++++++++++++++++++++")
            }
        }

        let (next_edge_idx, _) = self.search_tree
            .neighbors(idx)
            .map(|node_idx| {
                let edge_idx = self.parent_edge_idx(node_idx).unwrap();
                let edge = &self.search_tree[edge_idx];
                let exploration_stimulus = self.exploration_stimulus(&edge, n_b);
                (edge_idx, exploration_stimulus)
            })
            .max_by(
                |&(_, exploration_stimulus_a), &(_, exploration_stimulus_b)| {
                    exploration_stimulus_a
                        .partial_cmp(&exploration_stimulus_b)
                        .unwrap_or(::std::cmp::Ordering::Equal)
                },
            )
            .unwrap();

        let (_, next_node_idx) = self.search_tree.edge_endpoints(next_edge_idx).unwrap();
        next_node_idx
    }
    fn parent_node_idx(&self, idx: NodeIdx) -> Option<NodeIdx> {
        self.search_tree
            .neighbors_directed(idx, petgraph::Direction::Incoming)
            .nth(0)
    }
    fn parent_edge_idx(&self, idx: NodeIdx) -> Option<EdgeIdx> {
        self.parent_node_idx(idx)
            .and_then(|parent_idx| self.search_tree.find_edge(parent_idx, idx))
    }
    fn backup(&mut self, reward: f32, node_idx: NodeIdx) {
        if node_idx == self.root_idx {
            return;
        }
        let parent_edge_idx = self.parent_edge_idx(node_idx).unwrap();
        let parent_edge = self.search_tree.edge_weight_mut(parent_edge_idx).unwrap();

        parent_edge.visit_count += 1;
        parent_edge.total_value += reward;
        parent_edge.average_value = parent_edge.total_value / (parent_edge.visit_count as f32);

        let parent_idx = self.parent_node_idx(node_idx).unwrap();

        self.backup(1.0 - reward, parent_idx);
    }
    // Create an edge and a node for each possible move from this position
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

            let scale = 1.0 / total_probability; // Ok to divide by because at least one action must have non-zero probability
            for &mut (_, ref mut prior) in legal_actions.iter_mut() {
                *prior *= scale;
            }

            for (action, prior) in legal_actions {
                let new_state = game_expert.next(&self.search_tree[node_idx].state, &action);

                unsafe {
                    let leaf_idx = (*self_ptr)
                        .search_tree
                        .add_node(Node::new_unexpanded(new_state));

                    (*self_ptr).search_tree.add_edge(
                        node_idx,
                        leaf_idx,
                        Edge {
                            action,
                            prior,
                            visit_count: 0,
                            total_value: 0.0,
                            average_value: 0.5,
                        },
                    );
                }
            }
        }
        self.search_tree.node_weight_mut(node_idx).unwrap().expanded = true;
    }
}
