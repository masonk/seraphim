# Seraphim, an Alpha Zero-style game AI

Seraphim is a Rust library that efficiently solves the multiarmed bandit problem by exploring a search tree (such as a game tree) using the PUCT algorithm described in the original Alpha Go paper. The PUCT algorithm relies upon an expert policy (the `Inference`), that, given an abstract game state (a `serpahim::game::GameState`) can ascribe probabilities (or logits) to all possible actions (represented as `usize`).  Typically, the expert policy will be implemented as a machine learning model, such as a DNN (Deep Neural Network), but Seraphim is formally agnostic to this. Seraphim provides a reference implementation of `Inference` that is constructed from a Tensorflow SavedModel directory, and aspires to efficiently utilize the GPU and available CPU cores during search - but doesn't yet.

## TL;DR
Users implement Game[1] and Inference[2] and pass instances of those traits to Generate (for reinforcement learning) or Interactive (for a human to play a game against the computer).

[1] https://github.com/masonk/seraphim/blob/master/src/game.rs

[2] https://github.com/masonk/seraphim/blob/master/src/inference.rs

## The Reinforcement Learning Cycle
```
            ------------------ src/tictactoe/train.py <--------------------
            |                                                             |
            v                                                             |
    src/tictactoe/models/*                                                |
            |                                                             |
            v                                                             |
src/bin/generate_games.rs                                                 |
            |                                                             |
            -----------------> src/tictactoe/gamedata/*.tfrecord ----------
```

Seraphim uses an expert policy to search a game tree. After searching, Seraphim produces a record
of the best moves that it found. This record is a training example for the expert model - the model
trains itself to be more like the results of search, and the improved model is fed back to Seraphim
for subsequent searches.

Seraphim provides the searching. Your job as a consumer of Seraphim is to provide the description of the game's rules and an expert policy to use during search.


## The PUCT algorithm

For a given game state s, the PUCT algorithm takes a fixed number of samples  of the available actions (1600 in the paper; configurable in Seraphim by changing `search::SearchTreeOptions::readouts`). For each action it samples, a new state is generated, and applies the same logic again from this new state, recursively, until it reaches a terminal state.

When PUCT reaches a terminal state, it scores the game, and updates action values for every (state, action) pair that was visited on the way to the terminal state. In this way, the algorithm effectively plays 1600 games to the end each time it is asked to chose a action, always tending to favor better actions. It then choses to play the action it sampled most often in its search, subject to some noise in early actions, designed to produce variations in its play. You can customize the ply on which AGZ cools down its search by changing `search::SearchTreeOptions::tempering_point`. 

### Chosing the next action to sample

For each sample, PUCT choses from among all possible actions by always chosing the action, a, that maximises

Q(s,a) + U(s,a)

where Q(s,a) is the average value of action "a" from the current state, and

U(s,a) = cP(s,a)sqrt(N(s, b))/(1 + N(s,a))

is an exploration term that gives value to lesser explored nodes and to nodes that the expert prefers.

"c" is a hyperparameter that controls the tradeoff between exploitation (Q) and exploration (U). AGZ used c = 0.25 in the paper. You can configure any value for c in `search::SearchTreeOptions::cpuct`.

P(s,a) is the expert's prior belief that action a from state s is the best choice. It is a probability on [0,1]. This is the part of algorithm that must be supplied by the game expert.

N(s,b) is the number of times that the search has reached the current search state.

N(s,a) is the number of times that the search has chosen action a from state s. 
