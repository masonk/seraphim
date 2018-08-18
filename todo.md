- top priority

    - Search doesn't always find the right move, even if the next move is an immediate win.
        - Next step: extract the "improved hypotheses" out of search and display them in the interactive
        - Also display #samples taken for each next action
        - Tempering problem?
        - Another thought: there might be a bug in search that makes it stop looking at moves if they don't have the highest probability.

        Examples of search failures on an untrained model

            |o| |x|
            |x|x|o|
            | | |o|
            First (Computer) to move
            Next player has 0.5 probability of winning
            [0] 1 (0.11068193)
            [1] 6 (0.11068193)
            [2] 7 (0.11068193)

            |o|x|x|  <----- wrong, '6' would have won immediately
            |x|x|o|
            | | |o|
            Second (Human) to move
            Next player has 0.5 probability of winning
            [0] 6 (0.11011746)
            [1] 7 (0.11011746)

        Here's an example from a "trained" model (1m+ minibatches) that doesn't look very trained to me

            |x| | |
            | | | |
            |x|o| |
            Second (Human) to move
            Next player has 0.5 probability of winning
            [0] 1 (0.10471697)
            [1] 2 (0.112366766)
            [2] 3 (0.10471697)             <--- this is the correcct move. The net doesn't favor it, but doesn't hate it. Search should have been able to find this.
            [3] 4 (0.11834621)
            [4] 5 (0.10471697)
            [5] 8 (0.11780569)

                3
            |x| | |
            | |o| |
            |x|o| |
            First (Computer) to move
            Next player has 0.5 probability of winning
            [0] 1 (0.10236162)
            [1] 2 (0.11517083)
            [2] 3 (0.10236162)               <------ this move *wins the game immediately*; the net doesn't hate it, so search should always find it.
            [3] 5 (0.10236162)
            [4] 8 (0.117477946)

            |x|x| |            <----- lel  it played a different move - is tempering working correctly?
            | |o| |
            |x|o| |
            Second (Human) to move
            Next player has 0.5 probability of winning
            [0] 2 (0.11221403)
            [1] 3 (0.104785934)
            [2] 5 (0.104785934)
            [3] 8 (0.11434293)

    - The nets aren't training. The Adagrad model has almost no preference after a whole night of training. The Adam model has literally no preference.


    - summary metrics & checkpointing for Tensorboard
    - don't train '1' for the chosen move and '0' everywhere else, train search's prob distribution
        - after doing this, easy to add cross-entropy, equal weight to mse

    - L2 regularization 10e-4
    - loss = mse + cross_entropy + l2 regularization
    - minibatching + batch normalization
        - AZ paper used minibatches of 2048, this should be driven by the available hardware
    - minibatches were uniformly sampled from "the most recent 500,000 games of self-play" (I'm guessing each move was individually sampled?)
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
- push everything about game generation up into the main search modules
    - search:search, seraph::play
- try new models
- 1500 readouts

game:
- playing strength unit test
- training throughput benchmark
- multithreaded game generation
- legal_actions doesn't always quite add up to 1.0, due to floating point errors.

