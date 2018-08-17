- top priority
    - some sort of generalized debugging system for evaluating the strength of the game
    - summary metrics & checkpointing for Tensorboard

    - don't train '1' for the chosen move and '0' everywhere else, train search's prob distribution
        - after doing this, easy to add cross-entropy, equal weight to mse    
    - L2 regularization 10e-4
    - Adam optimizer
    - loss = mse + cross_entropy + l2 regularization
    - minibatching + batch normalization
        - AZ paper used minibatches of 2048, this should be driven by the available hardware
    - minibatches were uniformly sampled from "the most recent 500,000 games of self-play" (I'm guessing each move was individually sampled?)
    - every 1,000 minibatches, a new checkpoint of the model is evaluated
        - if it's better, it replace the previous model and new games are generated using it.
    - configure model + graph from a configuration file (?)
    - hash game_data and model based on graph and model
    - regularziation and batch training
    - add to_win to the hypothesis api
    - Debug TTT training problems:
        - Model capacity too low?
        - No regularization is probably screwing things up.
        - batching will add some regularization.
        - need many more training epochs?
contrib
    - TFExample package
search:
- implement a model evaluator
- optional dirichlet noise in the move selection function
- Separate read_and_apply into read() -> Action and advance(a: Action) so that tree search can be used to play interactively with an opponent, rather than just games of self-play
    - discard untaken edges in the search tree when advancing down a node
- Why does 200 readouts of TTT consistently underperform 100 readouts?
    - time cargo test expert::increasing_readouts -- --nocapture
- Multithread search
- (perf) Switch to a StableGraph for perf
- batching inference requests

ttt net:
- make sure weights are initialized with some random noise; it looks like they aren't by default.
- build GPU-enabled TF 1.6
- try new models
- 1500 readouts

game:
- playing strength unit test
- training throughput benchmark
- multithreaded game generation
- legal_actions doesn't always quite add up to 1.0, due to floating point errors.

