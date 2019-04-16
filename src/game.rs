use serde;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameStatus {
    InProgress,
    NullResult,
    Draw,
    LastPlayerWon,
    LastPlayerLost,
}

pub trait Action:
    std::hash::Hash
    + std::clone::Clone
    + std::fmt::Debug
    + serde::Serialize
    + serde::Deserialize
{}

impl<T> Action for T where
    T:
    std::hash::Hash
    + std::clone::Clone
    + std::fmt::Debug
    + serde::Serialize
    + serde::Deserialize
{}

pub trait State:
    std::cmp::Eq,
    + std::hash::Hash
    + std::clone::Clone
    + std::fmt::Debug
    + serde::Deserialize
    + serde::Serialize
{}

impl<T> State for T where 
T:
    std::cmp::Eq,
    + std::hash::Hash
    + std::clone::Clone
    + std::fmt::Debug 
    + std::default::Default
    + serde::Deserialize
    + serde::Serialize
{
    
}

pub trait Game<S, A> 
where S: State,
A: Action
{
    fn actions(&self, &State) -> Vec<Action>;

    // The given a state and an action on that state, the successor state
    fn successor(&self, &State, &Action) -> State;

    // The number of distinct possible actions in the game.
    // E.g., for TTT, there are 9. For go, there 19x19 + pass + resign = 363
    fn action_size(&mut self) -> usize;


}

