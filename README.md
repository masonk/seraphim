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

Seraphim provides the searching. Your job as a consumer of Seraphim is to provide the expert policy.  

The expert is abstracted by the `seraphim::search::GameExpert` trait. Implementations of the game expert are likely to use ML models to produce the required hypotheses, and the interface is designed with this use-case in mind, though strictly speaking, core seraphim does not know about ML models, and simply requires any implementation of a GameExpert in order to perform its search. In the paper, and in the included Tic Tac Toe example, the model is a fully connected DNN of customizable depth, written in TensorFlow.

Meanwhile, the GameExpert does know the rules of the game it is playing, and ascribes prior beliefs about the quality of possibles moves. It  is abstract over game state (S) and the possible game actions (A). Search does not know the rules of the game it is playing; it simply samples availables actions with reference to the probablity that the expert model ascribes to them.

## Installing Core Seraphim

The core Seraphim search library (in src/search) is agnostic to the implementation of the game expert. To use it as a library, you simply need to depend on the Seraphim crate and implement ` seraphim::search::GameExpert` and use it to initialize a new `seraphim::search::SearchTree` - which you can then sample to find next moves. Seraphim not yet published on crates.io; you can pull it and use it as a local crate if you want to use it now.

### Running the Tic Tac Toe example

This repo also ships with an example game engine that learns how to play Tic Tac Toe. Located in in src/tictactoe, this example uses TensorFlow to implement a DNN.

There are more system requirements to run the TTT trainer. Right now the example only works on Linux because it is set up to use nvidia-docker, which only works on Linux. (I welcome a patch to get this example training on Windows as well (with or without hardware acceleration)).

In total, you need
1. A Linux host.
2. A relatively recent version of the Docker engine. I have 18.09.1
3. A relatively recent version of Docker-compose. The composerfile requires a minimum Compose version of 2.3.
4. An nvidia NGC account to pull the nvidia docker image.
5. nvidia-docker2

The way this works is that you run a wrapper script, `./develop`. The first argument to `develop` is the name of the model you're workign with. The rest are arguments to `docker-compose`.

It's typically better to manage the action of the script in two console windows.

Run the train container and initialize a new model with random weights:
`./develop mymodel run train` 
`src/tictactoe/train.py mymodel --init`

Run the play container and generate games using the new model:
`./develop mymodel run play`
`cargo run --release --bin generate_games mymodel`

Allow some time to pass while game data accumulates, then in your train container:
`src/tictactoe/train.py mymodel`

You can play a game against mymodel with
`cargo run --release --bin interactive mymodel`

## Docker cheat sheet (from https://docs.docker.com/get-started/part2/)
docker build -t friendlyhello .  # Create image using this directory's Dockerfile
docker run -p 4000:80 friendlyhello  # Run "friendlyname" mapping port 4000 to 80
docker run -d -p 4000:80 friendlyhello         # Same thing, but in detached mode
docker container ls                                # List all running containers
docker container ls -a             # List all containers, even those not running
docker container stop <hash>           # Gracefully stop the specified container
docker container kill <hash>         # Force shutdown of the specified container
docker container rm <hash>        # Remove specified container from this machine
docker container rm $(docker container ls -a -q)         # Remove all containers
docker image ls -a                             # List all images on this machine
docker image rm <image id>            # Remove specified image from this machine
docker image rm $(docker image ls -a -q)   # Remove all images from this machine
docker login             # Log in this CLI session using your Docker credentials
docker tag <image> username/repository:tag  # Tag <image> for upload to registry
docker push username/repository:tag            # Upload tagged image to registry
docker run username/repository:tag                   # Run image from a registry


### Training tictactoe

`src/tictactoe` shows a full example of training a DNN using tensorflow. The training script is at `src/tictactoe/train.py`

You can run the training example with tracing enabled to see the whole algorithm at work.

```
source bin/activate
src/tictactoe/train.py --init my_model
cargo run --release generate_games my_model
```

Wait for searching script to generate some training examples (in tictactoe/gamedata/my_model/*.tfrecord). Then you can improve your naive model (which was initialized with random weights) by training on those examples.

```
src/tictactoe/train.py my_model
```


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

## Commands

source bin/activate
src/tictactoe/train.py my_model --init
cargo run --release --bin generate_games
cargo run --release --bin tfrecord_viewer
cargo run --release --bin debug
src/tictactoe/train/py my_model

