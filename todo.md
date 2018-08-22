Priority Queue:
    - Why do the nets learn to favor action 0 in turn 0? search likes playing 4 there
    - why don't the nets change their preference when moves are played? seems like they always have the same net output
        - are the nets training at all?
    - validation script
    - summary metrics & checkpointing for Tensorboard   
    - loss = mse + cross_entropy + l2 regularization

    - dirichlet noise (https://medium.com/oracledevs/lessons-from-alphazero-part-3-parameter-tweaking-4dceb78ed1e5)
    - L2 regularization 10e-4

    - if the nets still aren't training well after that, stop everything and debug it

    - hash game_data and model based on graph and model
    - add to_win to the hypothesis api
    - Why does 200 readouts of TTT consistently underperform 100 readouts?
        - time cargo test expert::increasing_readouts -- --nocapture
   
    - debug mode:
        - allow initialization from any state saved in a file
        - better (user-defined) parsing of next action in interactive games

Large Features:
    tictactoe: 
        - lift as much logic as possible into seraphim core
            - Librarize the Python half that all lives in train.py atm
        - tfrecord proto gencode should be in core
        - move to separate crate

    contrib
        - TFExample package

    interactive:
        - flesh out debugging & interactive game traits
        - history mode/undo/branch exploration
        - implement an example GUI

    search:
        - search benchmark
        - performance profile search
            - I guess there are large inefficients in how I'm using petgraph and how I'm searching for neighbors
        - implement a model evaluator (eternal tournamnent)
        - discard untaken edges in the search tree when advancing down a node
        - Multithread search

    python side:
        - many implicit contracts between the Rust and Python halves. Document them, librarize them
        - performance and strength benchmarking tools


