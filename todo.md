Priority Queue:
    - dirichlet noise (https://medium.com/oracledevs/lessons-from-alphazero-part-3-parameter-tweaking-4dceb78ed1e5)
    - L2 regularization 10e-4
    - loss = mse + cross_entropy + l2 regularization
    - summary metrics & checkpointing for Tensorboard
    - if the nets still aren't training well after that, stop everything and debug it
        - Why do the nets learn to favor action 0 in turn 0? search likes playing 4 there
    - hash game_data and model based on graph and model
    - add to_win to the hypothesis api
    - Why does 200 readouts of TTT consistently underperform 100 readouts?
        - time cargo test expert::increasing_readouts -- --nocapture
   
    - debug mode:
        - allow initialization from any state saved in a file
        - better (user-defined) parsing of next action in interactive games

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

