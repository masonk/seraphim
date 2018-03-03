/* This module uses Monte Carlo Tree Search informed by an expert policy to generate an improved analysis of the game state by sampling future states. This process is called "reading" when human players do it. The computer, however, does it quantitatively, and it reads to the end of the game before scoring a move.
*/
use petgraph;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    InProgress,
    TerminatedWithoutResult,
    LastPlayerWon,
    LastPlayerLost,
}

pub struct Hypotheses<Action> {
    actions: Vec<Action>, // Each legal action from a given state.
    // There must be at least one legal action at every non-terminal state.
    // If GameExpert::result returns InProgress, GameExpert::legal_action must return at least one action.
    beliefs: Vec<f32>, // the expert's prior belief that each action is the best move from the current position. Used in the expansion phase of the MCTS.
                       // these will be zipped together. In other words, the ith action corresponds to the ith belief.
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
    fn root(&self) -> State;

    // TODO: The AGZ paper minibatches the request for legal_actions
    // into batches 8. There should definitely be a way of batching these requests
    // This has to come after multi-threading the search, since threads block
    // while waiting for their batch to accumulate.
    fn legal_actions(&self, state: &State) -> (Vec<Action>, Vec<f32>);

    fn apply(&mut self, state: &State, action: &Action) -> State; // When MCTS choses a legal action from a particular state for the first time, it will call this function to expand a leaf node with a new state.

    fn to_win(&self, &State) -> f32; // What does think game expert think the *NEXT PLAYER'S* probability of winning the game is, from this position? This function will only be called on States that are GameResult::InProgress.
                                     // Todo: Find an interface that allows to_win to lock the thread until enough requests for expert policies have been made to fill a minibatch queue.

    // The prior probability that this action is the best next action
    // Used in the selection phase of the MCTS,
    fn prior_probability(&self, action: Action) -> f32;

    fn result(&self, &State) -> GameResult;
}

type NodeIdx = petgraph::graph::NodeIndex<petgraph::graph::DefaultIx>;
type EdgeIdx = petgraph::graph::EdgeIndex<petgraph::graph::DefaultIx>;
struct Edge<Action> {
    action: Action,     // The Action that this edge represents.
    prior: f32, // P(s, a). The prior probability of choosing this node, derived from the expert guess.
    visit_count: u32, // N(s, a) in the AGZ paper. How many times has the action that this edge represents been tried so far?
    total_value: f32, // W(s, a). The total value of an action over all the times it's been tried.
    average_value: f32, // Q(s, a). The average value of an action over all the times it's been tried. Equal to total_value / visit_count.
}

struct Node<State> {
    expanded: bool,
    parent_edge_idx: Option<EdgeIdx>,
    state: Option<State>,
    result: Option<GameResult>,
}
impl<State> Node<State> {
    fn new_unexpanded() -> Self {
        Node {
            expanded: false,
            parent_edge_idx: None,
            state: None,
            result: None,
        }
    }
}
pub struct SearchTree<G, State, Action>
where
    State: ::std::hash::Hash,
    G: GameExpert<State, Action>,
{
    game_expert: G,
    search_tree: petgraph::Graph<Node<State>, Edge<Action>>,
    cpuct: f32, // A constant determining the level of exploration
}

impl<G, State, Action> SearchTree<G, State, Action>
where
    State: ::std::hash::Hash,
    G: GameExpert<State, Action>,
{
    // pub fn get_improved_hypotheses(&mut self) -> Hypotheses<Action> {}

    // recursively follow the search to a terminal node (A node where GameResult is not InProgress),
    // then back up the tree, updating edge weights.
    fn proceed(&mut self, node_idx: NodeIdx, node: &mut Node<State>) {
        let self_ptr = self as *mut Self;

        if !node.expanded {
            self.expand(node_idx, node);
        }
        match node.result {
            Some(GameResult::InProgress) => unsafe {
                let next_idx = self.select_next_node(node_idx, node);
                let mut next_node = (*self_ptr).search_tree.node_weight_mut(next_idx).unwrap();

                return self.proceed(next_idx, &mut next_node);
            },
            _ => self.backup(node_idx, node),
        }
    }

    fn select_next_node(&self, idx: NodeIdx, node: &Node<State>) -> NodeIdx {
        /* in the AGZ paper
            we maximize
            Q(s, a) + U(s, a)
            
            where

            U(s, a) = cP(s,a)sqrt(Nb)/(1 + Na)

            where Na is the number of visits of to this edge,
            and Nb is the number of visits to the parent edge,
            c is "a constant determining the level of exploration".

        */
        let parent_edge = self.search_tree
            .edge_weight(node.parent_edge_idx.unwrap())
            .unwrap();
        let n_b = parent_edge.visit_count;
        let (next_edge_idx, _) = self.search_tree
            .neighbors(idx)
            .map(|node_idx| {
                let node = self.search_tree.node_weight(node_idx).unwrap();
                let edge_idx = node.parent_edge_idx.unwrap();
                let edge = self.search_tree.edge_weight(edge_idx).unwrap();
                let puct = self.cpuct * edge.prior * f32::sqrt(n_b as f32)
                    / ((1 + edge.visit_count) as f32);
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
    fn backup(&mut self, node_idx: NodeIdx, node: &mut Node<State>) {}
    // Create an edge and a node for each possible move from this position
    fn expand(&mut self, node_idx: NodeIdx, unexpanded_node: &mut Node<State>) {
        let self_ptr = self as *mut Self;
        unsafe {
            let parent_edge_idx = unexpanded_node.parent_edge_idx.unwrap();
            let (parent_idx, _) = (*self_ptr)
                .search_tree
                .edge_endpoints(parent_edge_idx)
                .unwrap();
            let parent = (*self_ptr).search_tree.node_weight(parent_idx).unwrap();
            let edge = (*self_ptr)
                .search_tree
                .edge_weight(parent_edge_idx)
                .unwrap();

            let state = self.game_expert
                .apply(&parent.state.as_ref().unwrap(), &edge.action);
            let result = self.game_expert.result(&state);

            match &result {
                &GameResult::InProgress => {}
                _ => return,
            }

            let (actions, priors) = self.game_expert.legal_actions(&state);

            unexpanded_node.state = Some(state);
            unexpanded_node.result = Some(result);
            let mut max_prior = 0.0;
            let mut max_edge: Option<EdgeIdx> = None;

            let mut second_max_prior = 0.0;
            let mut second_max_edge: Option<EdgeIdx> = None;
            for (action, prior) in actions.into_iter().zip(priors.into_iter()) {
                let leaf_idx = self.search_tree.add_node(Node::new_unexpanded());
                let edge_idx = self.search_tree.add_edge(
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
                if prior > max_prior {
                    max_prior = prior;
                    max_edge = Some(edge_idx);
                } else if prior > second_max_prior {
                    second_max_prior = prior;
                    second_max_edge = Some(edge_idx);
                }
            }
        }
    }
}
