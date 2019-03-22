Priority Queue:
    - Need a way to unify two board states that arrived at by different paths - may be algorithmically important
    - investigate j-curve in readouts:
        - 100 has more draws than 600
        - it starts going up again around 2000, nearing 100% draws by 10,0000
        
    - loss = mse + cross_entropy + l2 regularization
        - L2 regularization 10e-4
    - add to_win to the hypothesis api


    - hardware accelerate inference
        - debug slow inference perf on gpu
        - batching?
    - summary metrics & checkpointing for Tensorboard   
        - accuracy metric
    - benchmarks
        - profile performance
    - unit tests    
        - Do the posterior possibilities add to one?
        - testing framework for implementors
    - auto tournament for model comparison
    - multithread search
        - replace petgraph with a custom lockless search tree

    - debug mode:
        - allow initialization from any state saved in a file
        - better (user-defined) parsing of next action in interactive games

Large Features:
    tictactoe: 
        - lift as much logic as possible into seraphim core
            - Librarize the Python half that all lives in train.py atm
        - tfrecord proto gencode should be in core
        - move TTT to separate crate

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


