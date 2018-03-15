- Why doesn't MCTS find the right move in this test: 
cargo test expert::search_blocks_immediate_loss -- --nocapture

- Clear up the heisenbug where sometimes State is full of o's
- Add ability to advance the search tree from outside (e.g., if a different player plays a move, need to start analyzing from the new state)
- Get the debugger working 