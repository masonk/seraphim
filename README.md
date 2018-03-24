# Seraphim, an Alpha Zero-style game AI

Seraphim is a monte carlo tree search algorithm that uses the AGZ variant of the PUCT algorithm ("primary upper confidence tree") to chose its next action.

This variant of the algorithm relies on an _expert policy_ that, given a state, has a belief about which legal action is most likley to be the best one.

The tree search is more likely to sample actions that the expert policy believes are probably the best ones. If the expert policy is good, then the MCTS will avoid lousy actions and spend most of its time examining good lines of play.

## TLDR

Your job as a consumer of Seraphim is to implement seraphim::search::GameExpert<S, A> for the game you want Seraphim to learn. The GameExpert knows the rules of the game and has prior beliefs about which move is best from every position. seraphim::search runs a MCTS on each game state by using the MCTS to supply legal next actions and its prior beliefs about each action. 

## The PUCT algorithm

For a given game state s, the PUCT algorithm takes a fixed number of samples  of the available actions (1600 in the paper, configurable in Seraphim). For each action it samples, a new state is generated, and applies the same logic again from this new state, recursively, until it reaches a terminal state.

When PUCT reaches a terminal state, it scores the game, and updates action values for every (state, action) pair that was visited on the way to the terminal state. In this way, the algorithm effectively plays 1600 games to the end each time it is asked to chose a action, always tending to favor better actions. It then choses to play the action it sampled most often in its search, subject to some noise in order to produce variations in its play. 

### Chosing the next action to sample

For each sample, PUCT choses from among all possible actions by always chosing the action, a, that maximises

Q(s,a) + U(s,a)

where Q(s,a) is the exploitation term, where the average value of action "a" from the current state, and

U(s,a) = cP(s,a)sqrt(N(s, b))/(1 + N(s,a))

is an exploration term that gives weight to lesser explored nodes and to nodes that the expert prefers.

"c" is a hyperparameter that controls the tradeoff between exploitation (Q) and exploration (U). AGZ used c = 0.25 in the paper.

P(s,a) is the expert's prior belief that action a from state s is the best choice. It is a probability on [0,1]. This is the part of algorithm that must be supplied by the game expert.

N(s,b) is the number of times that the search has reached the current search state.

N(s,a) is the number of times that the search has chosen action a from state s. 

## The Game Expert

The puct algorithm crucially hinges on the ability of an outside expert to supply quality priors about which actions to explore. In the AGZ paper, this was the job of the AGZ DNN.

In Seraphim, the GameExpert is a generic trait which is to be implemented for each individual game. The GameExpert trait encapsulates not only the expert policy, but also the rules of the game itself. The search algorithm has no specific knowledge of any game rules, and operates entirely by asking the GameExpert for lists of next actions and next states and the final result.

## Getting started

src/tictactoe/mod.rs contains an implementation of a GameExpert and its unit tests show examples of self-play of TicTacToe using a very simple expert policy (giving every legal move the same weight). Since TTT is a game with a small state space, the MCTS algorith alone is usually, but not always, able to find the best move from any position.