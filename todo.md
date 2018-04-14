search:
- update the algorithm to also produce an improved to_win probability    
    - then update TTT to train that
- separate move generation and training - can run the two halfs asynchronously and on multiple hosts simultaneously.
- Dirichlet noise in the move selection function
- Separate read_and_apply into read() -> Action and advance(a: Action) so that tree search can be used to play interactively with an opponent, rather than just games of self-play
    - discard untaken edges in the search tree when advancing down a node
- Why does 200 readouts of TTT consistently underperform 100 readouts?
    - time cargo test expert::increasing_readouts -- --nocapture
    - Make sure the selected action is best from the PoV of the *current player*
- (perf) Switch to a StableGraph for perf?
- Multithread search?
- batching inference requests

ttt net:
- summary metrics & checkpointing for Tensorboard
- continuous training mode
- make sure weights are initialized with some random noise; it looks like they aren't by default.
- build GPU-enabled TF 1.6
- try new models

game:
- playing strength unit test
- training throughput benchmark
- multithreaded game generation
- legal_actions doesn't always quite add up to 1.0, due to floating point errors.

