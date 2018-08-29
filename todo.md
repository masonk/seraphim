Priority Queue:
    - summary metrics & checkpointing for Tensorboard   
        - accuracy metric
    - add to_win to the hypothesis api
    - unit tests    
        - Do the posterior possibilities add to one?
        - testing framework for implementors
    - auto tournament for model comparison
    - loss = mse + cross_entropy + l2 regularization
    - dirichlet noise (https://medium.com/oracledevs/lessons-from-alphazero-part-3-parameter-tweaking-4dceb78ed1e5)
    - L2 regularization 10e-4

    - Why does 200 readouts of TTT consistently underperform 100 readouts?
        - time cargo test expert::increasing_readouts -- --nocapture
   
    - debug mode:
        - show loss for each net prediction (search - net)
        - allow initialization from any state saved in a file
        - better (user-defined) parsing of next action in interactive games

Large Features:
    tictactoe: 
        - lift as much logic as possible into seraphim core
            - Librarize the Python half that all lives in train.py atm
        - tfrecord proto gencode should be in core
        - move to separate crate

    contrib
        - Contribute TFReader

    interactive:
        - flesh out debugging & interactive game traits
        - history mode/undo/branch exploration
        - implement an example GUI

    search:
        - search benchmark
        - performance profile search
            - I guess there are large inefficiencies in how I'm using petgraph and how I'm searching for neighbors
            - I should implement a high concurrency tree structure
        - implement a model evaluator (eternal tournamnent)
        - discard untaken edges in the search tree when advancing down a node
        - Multithread search

    python side:
        - many implicit contracts between the Rust and Python halves. Document them, librarize them
        - performance and strength benchmarking tools


