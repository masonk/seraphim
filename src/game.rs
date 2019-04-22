#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// TODO: Generalize game status to per-player q values
// This will make it easier to model multiplayer games that are not winner-takes-all
// and non-zero-sum games
pub enum GameStatus {
    InProgress,
    NullResult,
    Draw,
    LastPlayerWon,
    LastPlayerLost,
}

impl std::default::Default for GameStatus {
    fn default() -> Self {
        GameStatus::InProgress
    }
}

pub trait GameState:
    std::cmp::Eq + std::hash::Hash + std::clone::Clone + std::fmt::Debug + std::default::Default
{
    // Each training example will be recorded as this bytestring (representing the game state)
    // concatenated to packed f32s representing the trainable probabilities
    // fn to_feature(self) -> Vec<u8>;
    // TODO: When GATs land, this should be replaced with a type constructor that allows
    // reference types
    fn feature_bytes(&self) -> Vec<u8>;
}

pub trait Game {
    type State: GameState;

    // How many different actions is it possible to sample in the game?
    // E.g. for go, it is 19x19 + resign + pass
    fn action_count(&self) -> usize;

    // All the action indexes that are legal for a given State
    // nonlegal actions will be forced to 0 probability by the search engine
    fn legal_actions(&self, state: &Self::State) -> Vec<bool>;

    // Ggiven a state and an action on that state, the successor state
    fn successor(&self, state: &Self::State, action: usize) -> Self::State;

    fn status(&self, state: &Self::State) -> GameStatus;

    // All the symmetrical training examples, these will be packed into the training example output
    fn symmetries(&self, _hypotheses: &Vec<f32>) -> Vec<(Vec<u8>, Vec<f32>)> {
        vec![]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Humanity {
    Human,
    Computer,
}

pub trait Player: std::fmt::Display + std::fmt::Debug + Sized {
    fn humanity(&self) -> Humanity;
}

pub trait AsciiInteractive: Game {
    type Player: self::Player;
    // Returns the Player that makes the next action
    fn to_play(&self, _: &Self::State) -> &Self::Player;

    // Prompt a human player to enter their next action on stdin and returns that action
    fn prompt(&self, _: &Self::State) -> usize;
}

// Convenience impls for two players named Black and White
#[derive(Debug, Clone, Copy)]
pub enum BlackWhite {
    Black,
    White,
}
impl std::fmt::Display for BlackWhite {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            BlackWhite::Black => write!(f, "{}", "black"),
            BlackWhite::White => write!(f, "{}", "white"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlackWhitePlayer {
    id: BlackWhite,
    humanity: Humanity,
}

impl std::fmt::Display for BlackWhitePlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.id)
    }
}
impl Player for BlackWhitePlayer {
    fn humanity(&self) -> Humanity {
        self.humanity
    }
}
