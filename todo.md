- Implement a game expert with a trainable net
    - Handle illegal moves. Restribute probability to legal moves by scaling.
    - Separate read_and_apply into read() -> Action and apply(a: Action) so that tree search can be used to player interactively with an opponent, rather than just games of self-play
    - separate move generation and training - can run the two halfs asynchronously and on multiple hosts simultaneously.
    - make sure weights are initialized with some random noise
    - implement summary metrics for Tensorboard
    

- Why does 200 readouts of TTT consistently underperform 100 readouts?
time cargo test expert::increasing_readouts -- --nocapture

- Make sure the selected action is best from the PoV of the *current player*

- Add ability to advance the search tree from outside (e.g., if a different player plays a move, need to start analyzing from the new state)

- Get the debugger working 

- (perf) Switch to a StableGraph for perf?
- Multithread search
