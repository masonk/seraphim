/* This module implements informed Monte Carlo tree search. "Informed" means that the search is guided by an expert policy that ascribes Bayesian prior probabilities to question of whether each possible next action is the best one.*/
use petgraph;
use rand;
use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameResult {
    InProgress,
    TerminatedWithoutResult,
    LastPlayerWon,
    LastPlayerLost,
}

pub struct Hypotheses<Action> {
    pub actions: Vec<Action>, // Each legal action available from the a certain State
    pub move_probabilities: Vec<f32>, // The prior probability that each action is the best. These should be in the same order as the corresponding actions Vec
    pub to_win: f32,                  // The probability that the next player will win the game
}

pub struct SearchResult<Action> {
    next_move: Action,              // the chosen Action
    hypotheses: Hypotheses<Action>, // for training the expert
}
/* 
The expert that guides the MCTS search is abstracted by the GameExpert trait, which users of this library are to implement.  The GameExpert knows the rules of the game it's playing, and also has bayesian prior beliefs about which moves are best to play and the probability that the next player will ultimately win the game.

This trait is meant to be implemented by a consumer of the library. 

State is a state of the game. It will be created by apply()ing an Action to an existing State or by calling root() to get the root state. For each (state, action) pair, expert.apply(&state, &action) will only be called one time.

*/
pub trait GameExpert<State, Action>
where
    State: ::std::hash::Hash,
{
    // TODO: The AGZ paper minibatches the request for expert policies
    // into batches of 8 hypotheses. There should be a way of batching these requests
    // This has to come after multi-threading the search, since threads block
    // while waiting for their batch to accumulate.
    fn hypotheses(&mut self, state: &State) -> Hypotheses<Action>;

    fn next(&mut self, &State, &Action) -> State; // When MCTS choses an action for the first time, it will call this function to obtain the new state. Used during the MCTS leaf expansion step.

    // What is the search::GameResult of this game? Search continues until it reaches a terminal (non-InProgress) state, and then backpropagates based on the final result of the game.
    // and then
    fn result(&self, &State) -> GameResult;

    // TODO: is this function necessary, or should GameExperts just train themselves to be more likely to play the chosen move?
    // // train() will be called once per move that was actually made in the game.
    // // with the improved Hypotheses that are a result of the tree search.
    // // It is up to the game expert to use this information to train its underlying
    // // model.
    // fn train(&mut self, &State, &Hypotheses<Action>);
}

type NodeIdx = petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>;
type EdgeIdx = petgraph::graph::EdgeIndex<petgraph::graph::DefaultIx>;

#[derive(Debug)]

struct Edge<Action>
where
    Action: PartialEq + ::std::fmt::Debug,
{
    action: Action,     // The Action that this edge represents.
    prior: f32, // P(s, a). The prior probability of choosing this node, derived from the expert guess.
    visit_count: u32, // N(s, a) in the AGZ paper. How many times has the action that this edge represents been tried so far?
    total_value: f32, // W(s, a). The total value of an action over all the times it's been tried.
    average_value: f32, // Q(s, a). The average value of an action over all the times it's been tried. Equal to total_value / visit_count.
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
    pub cpuct: f32, // a constant determining the tradeoff between exploration and exploitation; .25 in the AGZ paper. Higher numbers bias the search towards less-explored nodes, lower numbers bias the search towards more promising nodes.
    pub readouts: u32, // how many games to play when searching for a single move; 1600 in the AGZ paper
    pub tempering_point: u32, // how many plys should progress until we lower the temperature of move selection from 1 to ~0. 30 in the AGZ paper
}
impl SearchTreeOptions {
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
    State: ::std::hash::Hash + ::std::fmt::Debug + ::std::fmt::Display,
    Action: PartialEq + ::std::hash::Hash + ::std::fmt::Debug,
{
    rand: rand::ThreadRng,
    search_tree: petgraph::stable_graph::StableGraph<Node<State>, Edge<Action>>,
    ply: u32,          // how many plys have been played at the root_idx
    root_idx: NodeIdx, // cur
    options: SearchTreeOptions,
}

impl<State, Action> SearchTree<State, Action>
where
    State: ::std::hash::Hash + ::std::fmt::Debug + ::std::fmt::Display,
    Action: PartialEq + ::std::hash::Hash + ::std::fmt::Debug,
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

    pub fn read_and_apply(&mut self, game_expert: &mut GameExpert<State, Action>) -> Action {
        /*
        Read the board by playing out a number of games according to the PUCB MCTS algorithm.

        Select a next move using the AGZ annealing method:

            At the end of the search AGZ selects a move to play
            in the root position proportional to its exponentiated visit count
            p(a|s_0) = N(s_0, a)^{1/T} / \sum{b}N(s_0, b)^{1/T},  
            where T is a temperature parameter. The search tree is reused at subsequent
            timesteps: the played action becomes the new root node, and other nodes are discarded.

            AGZ resigns if its root value and best child value are lower than a threshhold.

        Apply that selection to the search tree by advancing down the tree, and return the picked action so that the GameExpert will know what was picked.

        The GameExpert should train itself to be more likely to pick the same value that the tree search picked.
        */
        for _ in 0..self.options.readouts {
            self.read_one(self.root_idx, game_expert);
        }

        let visits: Vec<(u32, EdgeIdx, NodeIdx)> = self.search_tree
            .neighbors(self.root_idx)
            .map(|n| (self.parent_edge_idx(n).unwrap(), n))
            .map(|(e, n)| {
                let edge = self.search_tree.edge_weight(e).unwrap();
                (edge.visit_count, e, n)
            })
            .collect();

        let total_visit_count: u32 = visits.iter().map(|&(count, _, _)| count).sum();
        if self.ply < self.options.tempering_point {
            let rand: f32 = self.rand.gen_range(0.0, 1.0);
            let mut cum_prob = 0.0;

            for (count, edge_idx, node_idx) in visits {
                let prob: f32 = (count as f32) / (total_visit_count as f32);
                cum_prob += prob;
                if cum_prob > rand {
                    self.ply += 1;
                    self.root_idx = node_idx;
                    // TODO: Remove all nodes that are made unreachable due to advancing down the tree.
                    let edge = self.search_tree.remove_edge(edge_idx).unwrap();
                    return edge.action;
                }
            }
        } else {
            // After 30 plys, find the move with the highest visit count
            let (selected_edge, selected_node) = self.search_tree
                .neighbors(self.root_idx)
                .map(|child_idx| (self.parent_edge_idx(child_idx).unwrap(), child_idx))
                .max_by_key(|&(edge_idx, _)| self.search_tree[edge_idx].visit_count)
                .unwrap();
            //todo: Discard unreachable edges
            trace!("{:?} {:?}", selected_edge, selected_node);
            self.ply += 1;
            self.root_idx = selected_node;
            let edge = self.search_tree.remove_edge(selected_edge).unwrap();
            return edge.action;
        }
        panic!("Didn't select a next action.");
    }

    // update the search tree by applying an action.
    pub fn apply(&mut self, action: Action) {
        let next_node_idx = self.search_tree
            .neighbors(self.root_idx)
            .find(|node_idx| {
                self.search_tree[self.parent_edge_idx(*node_idx).unwrap()].action == action
            })
            .unwrap();
        self.ply += 1;
        self.root_idx = next_node_idx;
        // TODO: delete all nodes and edges that are not reachable from root
    }
    // recursively follow the search to a terminal node (A node where GameResult is not InProgress),
    // then back up the tree, updating edge weights.
    fn read_one(&mut self, node_idx: NodeIdx, game_expert: &mut GameExpert<State, Action>) {
        let self_ptr = self as *mut Self;
        unsafe {
            let node = (*self_ptr).search_tree.node_weight_mut(node_idx).unwrap();
            let result = game_expert.result(&node.state);
            match result {
                GameResult::InProgress => {
                    if !node.expanded {
                        self.expand(node_idx, game_expert);
                    }
                    let next_idx = self.select_next_node(node_idx, game_expert);

                    return self.read_one(next_idx, game_expert);
                }
                _ => {
                    match result {
                        GameResult::LastPlayerLost => self.backup(-1.0, node_idx),
                        GameResult::LastPlayerWon => self.backup(1.0, node_idx),
                        GameResult::TerminatedWithoutResult => self.backup(0.0, node_idx),
                        _ => {
                            warn!("Tried to backup some unknown terminal state. Algorithm error. ")
                        }
                    };
                }
            }
        }
    }

    fn select_next_node(&self, idx: NodeIdx, game_expert: &mut GameExpert<State, Action>) -> NodeIdx {
        /* in the AGZ paper
            The next node is the node with a that maximizes
            Q(s, a) + U(s, a)
            
            where

            U(s, a) = cP(s,a)sqrt(Nb)/(1 + Na)

            Na is the number of visits of to this edge,
            Nb is the number of visits to the parent edge,
            c is "a constant determining the level of exploration".

        */
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
        let (next_edge_idx, _) = self.search_tree
            .neighbors(idx)
            .map(|node_idx| {
                let edge_idx = self.parent_edge_idx(node_idx).unwrap();

                let edge = &self.search_tree[edge_idx];
                let puct = self.options.cpuct * edge.prior * f32::sqrt(n_b as f32)
                    / ((1 + edge.visit_count) as f32)
                    + edge.average_value;
                (edge_idx, puct)
            })
            .max_by(|&(_, puct_a), &(_, puct_b)| {
                puct_a
                    .partial_cmp(&puct_b)
                    .unwrap_or(::std::cmp::Ordering::Equal)
            })
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

        self.backup(reward * -1.0, parent_idx);
    }
    // Create an edge and a node for each possible move from this position
    fn expand(&mut self, node_idx: NodeIdx, game_expert: &mut GameExpert<State, Action>) {
        {
            let node = &self.search_tree[node_idx];
            let state = &node.state;
            let result = game_expert.result(state);

            match &result {
                &GameResult::InProgress => {}
                _ => {
                    return;
                }
            }
        }
        {
            let self_ptr = self as *mut Self;
            let Hypotheses {
                actions,
                move_probabilities,
                ..
            } = game_expert.hypotheses(&self.search_tree[node_idx].state);

            let states: Vec<(Action, State)> = actions
                .into_iter()
                .map(|action| {
                    let new_state = game_expert.next(&self.search_tree[node_idx].state, &action);
                    (action, new_state)
                })
                .collect();

            for ((action, new_state), prior) in
                states.into_iter().zip(move_probabilities.into_iter())
            {
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
                            average_value: 0.0,
                        },
                    );
                }
            }
        }
        self.search_tree.node_weight_mut(node_idx).unwrap().expanded = true;
    }
}
