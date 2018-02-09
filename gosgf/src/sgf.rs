// #[derive(Debug)]
// pub struct SgfDocument {
//     games: Vec<GameTree>,
// }

// #[derive(Debug)]
// pub struct GameTree {
//     pub komi: f64,
//     pub size: usize, // e.g. 19
//     pub handicap: usize,
//     pub sequence: Vec<Node>,
//     pub children: Vec<GameTree>,
// }

// #[derive(Debug)]
// pub struct Node {
//     pub properties: HashMap<String, String>, // all raw prop parses for this node
// }

// #[derive(Debug)]
// pub struct Property {
//     pub ident: String,
//     pub values: Vec<PropValue>,
// }

// #[derive(Debug)]
// pub struct PropValue {}

// pub struct Compose(ValueType, ValueType);
// pub enum Double {
//     Simple,
//     Emphasized,
// }
// pub enum Color {
//     Black,
//     White,
// }
// pub enum ValueType {
//     None,
//     Integer(i64),
//     Real(f32),
//     Double(f64),
//     Color,
//     FormattedText(String),
//     SimpleText(String),
//     RawValue(String),
// }

// /*
//   Collection = GameTree { GameTree }
//   GameTree   = "(" Sequence { GameTree } ")"
//   Sequence   = Node { Node }
//   Node       = ";" { Property }
//   Property   = PropIdent PropValue { PropValue }
//   PropIdent  = UcLetter { UcLetter }
//   PropValue  = "[" CValueType "]"
//   CValueType = (ValueType | Compose)
//   ValueType  = (None | Number | Real | Double | Color | SimpleText |
// 		Text | Point  | Move | Stone)
//         */
