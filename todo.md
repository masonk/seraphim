- Implement a game expert with a trainable net
    - Pass in GameExpert to the SearchTree, don't own the GameExpert
    - Save the model to disk & implementing summary metrics for Tensorboard
    - move GameExpert training code into tictactoe::mod (from train_tictactoe_simple)
    - Separate read_and_apply into read() -> Action and apply(a: Action)
    - Have the GameExpert use the net to build its Hypotheses

- Why does 200 readouts of TTT consistently underperform 100 readouts?
time cargo test expert::increasing_readouts -- --nocapture

- Make sure the selected action is best from the PoV of the *current player*

- Add ability to advance the search tree from outside (e.g., if a different player plays a move, need to start analyzing from the new state)

- Get the debugger working 

- (perf) Switch to a StableGraph for perf?
- Multithread search
