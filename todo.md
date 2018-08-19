Priority Queue:
    - debug mode:
        - separate "search" and "train"
        - allow user to chose the action for every ply
        - "enter" just choses the computer's move
        - allow initialization from any state saved in a file
        - better (user-defined) parsing of next action in interactive games

    - dirichlet noise (https://medium.com/oracledevs/lessons-from-alphazero-part-3-parameter-tweaking-4dceb78ed1e5)
    - summary metrics & checkpointing for Tensorboard
    - cross-entropy in loss function, equal weight to mse
    - L2 regularization 10e-4
    - loss = mse + cross_entropy + l2 regularization
    - batch normalization
        - AZ paper used minibatches of 2048, this should be driven by the available hardware
    - hash game_data and model based on graph and model
    - add to_win to the hypothesis api
    - Why does 200 readouts of TTT consistently underperform 100 readouts?
        - time cargo test expert::increasing_readouts -- --nocapture
   
Large Features:
    contrib
        - TFExample package

    interactive:
        - flesh out debugging & interactive game traits
        - history mode/undo/branch exploration

    search:
        - implement a model evaluator (eternal tournamnent)
            - discard untaken edges in the search tree when advancing down a node
        - Multithread search
        - (perf) Switch to a StableGraph for perf
        - get rid of the awful inefficient way that parents and children are related in Petgraph rn

    ttt net:
        - Lift as much code as possible into a library
        - Python API or library for the training half that all lives in train.py atm

    game:
        - playing strength unit test
        - training throughput benchmark
        - multithreaded game generation

