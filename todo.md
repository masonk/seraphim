- top priority
    - add to_win to the hypothesis api
    - Debug TTT training problems:
        - Model capacity too low?
        - No regularization is probably screwing things up.
        - batching will add some regularization.
        - need many more training epochs?

search:

- optional dirichlet noise in the move selection function
- Separate read_and_apply into read() -> Action and advance(a: Action) so that tree search can be used to play interactively with an opponent, rather than just games of self-play
    - discard untaken edges in the search tree when advancing down a node
- Why does 200 readouts of TTT consistently underperform 100 readouts?
    - time cargo test expert::increasing_readouts -- --nocapture
- Multithread search
- (perf) Switch to a StableGraph for perf
- batching inference requests

ttt net:
- regularziation and batch training
- summary metrics & checkpointing for Tensorboard
- make sure weights are initialized with some random noise; it looks like they aren't by default.
- build GPU-enabled TF 1.6
- try new models
- 1500 readouts

game:
- playing strength unit test
- training throughput benchmark
- multithreaded game generation
- legal_actions doesn't always quite add up to 1.0, due to floating point errors.

