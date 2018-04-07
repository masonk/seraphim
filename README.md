# Seraphim, an Alpha Zero-style game AI

[TOC]
Seraphim is a monte carlo tree search algorithm that uses the AGZ variant of the PUCT algorithm ("primary upper confidence tree") to chose its next action.

This variant of the algorithm relies on an _expert policy_ that, given a state, has a belief about which legal action is most likley to be the best one.

The tree search is more likely to sample actions that the expert policy believes are probably the best ones. If the expert policy is good, then the MCTS will avoid lousy actions and spend most of its time examining good lines of play.


## TLDR

Your job as a consumer of Seraphim is to implement `seraphim::search::GameExpert<S, A>` for the game you want Seraphim to learn. The GameExpert knows the rules of the game and has prior beliefs about which move is best from every position. seraphim::search runs a MCTS on each game state by querying the `GameExpert` to supply legal next actions and its prior beliefs about each action. 

## Installation

### Non-Cargo dependencies
#### IMPORTANT NOTE

> You'll have to build Tensorflow from source, with GPU acceleration enabled, in order to
> run the nets or build your own nets.
> This is an involved process and the only officially supported platforms for this are Ubuntu 16.04 and MacOS.
> I tried to build this on Windows and it was such a pain that I found it easier to download Ubuntu and dual boot into Ubuntu just to develop this net.

- [Nightly Rust](https://github.com/rust-lang-nursery/rustup.rs/#other-installation-methods)
- C linker, such as `gcc`
- openssl dev package

    On Ubuntu:

    ```
    apt install libssl-dev
    ```


- [Python 3](https://www.tensorflow.org/install/install_linux#installing_with_virtualenv) 
   
   This is to run the Python scripts that build the tensorflow graphs used by the game experts. You can set up the seraphim repo as a Python3 virtualenv root by following the linked instructions.

    e.g.
   
   ```
   sudo apt install python3-pip python3-dev python-virtualenv
   cd seraphim
   virtualenv --system-site-packages -p python3 .
   source bin/activate
   ```

- [TensorFlow for python](https://www.tensorflow.org/install/install_linux#installing_with_virtualenv)
 
   (enable your virtualenv first, if you're using one, then install the pip package)
   
   ```
   pip3 install --upgrade tensorflow-gpu
   ```

- [GPU-accelerated libtensorflow.so](https://github.com/tensorflow/rust#manual-tensorflow-compilation)
   
   Bite the bullet and build tensorflow core from scratch with GPU acceleration enabled. This is an involved process, but there's no point in running the nets if you aren't GPU accelerated. This substep involves installing Cuda **9.0** (not 9.1), CuDNN **7.0** (not 7.1), and Ubuntu **16.04** (not 17 or 18).



### Cargo
This is a Cargo project, built on nightly rust. `cargo` is Rust's module and build system, equivalent to node's `npm`.

This project requires nightly rust, because it uses feature gates. Use rustup to switch to a nightly toolchain if you haven't already, then run

`cargo build`
`cargo test`

to verify the installation. A few tests might fail, such as `tictactoe::expert::increasing_readouts_improves_play`. But everything should build.

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

## The Game Expert

The puct algorithm crucially hinges on the ability of an outside expert to supply quality priors about which actions to explore. In the AGZ paper, this was the job of the AGZ DNN.

In Seraphim, the GameExpert is a generic trait which is to be implemented for each individual game. The GameExpert trait encapsulates not only the expert policy, but also the rules of the game itself. The search algorithm has no specific knowledge of any game rules, and operates entirely by asking the GameExpert for lists of next actions and next states and the final result.

## Getting started

[Tic Tac Toe](src/tictactoe/mod.rs) contains an implementation of a GameExpert and its unit tests show examples of self-play of TicTacToe using a very simple expert policy (giving every legal move the same weight). Since TTT is a game with a small state space, the MCTS algorith alone is usually, but not always, able to find the best move from any position.