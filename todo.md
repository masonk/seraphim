Priority Queue:
    - batching inferences 
        - A separate thread that batches inferences on a 1-10ms timeout
        - State implements Hash and we cache the result of inferences
        - 128-1024 tasks, each running its own game, on a threadpool
        - Use Actix, InferenceBatcher is an Actor, spawn 1024 "play a game" tasks on a threadpool
    - too much Dirichlet?
    - Game API v2:
        - separate Expert from Game
        - Automatically implement GameExpert for game and make generate_games work for any game
        pub struct Hypotheses<Action> {
            pub actions: Vec<Action>,
            pub priors: Vec<f32>,
            pub q: f32,
        }
        struct TrainingExample<S, A> {
            state: S,
            hypotheses: Hypotheses<A>
            q: f32
        }

        trait Game<S, A> {
            symmetries(&TrainingExample<S>) -> Vec<TrainingExample<S>>
        }

        State and Action implement Serialize, Deserialize to the format used in training

        - 
    - Fill in q values at the end of the game
    - Emit all symmmetries of a game as examples



    - lift as much logic as possible into seraphim core
        - Librarize the Python half that all lives in train.py atm
    - tfrecord proto gencode should be in core
    - move TTT to separate crate
    - benchmarks
        - profile performance
        - triage performance

    - investigate j-curve in readouts:
        - 100 has more draws than 600
        - it starts going up again around 2000, nearing 100% draws by 10,0000
        
    - loss = mse + cross_entropy + l2 regularization
        - L2 regularization 10e-4

    - hardware accelerate inference
        - debug slow inference perf on gpu
        - batching?
    - summary metrics & checkpointing for Tensorboard   
        - accuracy metric

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
    contrib
        - Contribute TFReader

    interactive:
        - flesh out debugging & interactive game traits
        - history mode/undo/branch exploration
        - implement an example GUI

    search:
        - search benchmark
        - discard untaken edges in the search tree when advancing down a node
        - Multithread search

    python side:
        - many implicit contracts between the Rust and Python halves. Document them, librarize them
        - performance and strength benchmarking tools


