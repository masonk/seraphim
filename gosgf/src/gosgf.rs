use std::collections::HashMap;
pub type GoCollection = Vec<GameTree>;

pub struct Point(pub char, pub char);

pub enum PointColor {
    Black,
    White,
    Empty,
}

pub enum PlayerColor {
    Black,
    White,
}

pub struct Stone {
    color: PointColor,
    point: Point,
}

pub enum Move {
    /* 
    From the implementation guide (http://www.red-bean.com/sgf/ff5/m_vs_ax.htm), referencing the spec (http://www.red-bean.com/sgf/go.html#types)
    How to execute a move
    When a B (resp. W) property is encountered, a stone of that color is placed on the given position (no matter what was there before).
    Then the application should check any W (resp. B) groups that are adjacent to the stone just placed. If they have no liberties they should be removed and the prisoner count increased accordingly.
    Then, the B (resp. W) group that the played stone belongs to should be checked for liberties, and if it has no liberties, it should be removed (suicide) and the prisoner count increased accordingly.
    Lastly, the move number should be increased by one. */
    Of(Stone),
    Pass, //

    /* From the spec (http://www.red-bean.com/sgf/ff5/m_vs_ax.htm):
    Add (resp. clear) black/white stones to the board. This can be used to set up positions or problems. Adding (resp. clearing) is done by 'overwriting' the given point with black/white/empty stones. It doesn't matter what was there before. Adding a stone must not trigger any rule specific actions, e.g. in Go GM[1]: no prisoners nor any other captures (e.g. suicide). Thus it's possible to create illegal board positions. */
    Add(Stone),
    SetMoveNumber(isize),
}
pub struct GameTree {
    komi: f64,
    size: usize, // e.g. 19
    handicap: usize,
    sequence: Vec<Node>,
    children: Vec<GameTree>,
}

pub struct Node {
    pub properties: HashMap<String, String>, // all raw prop parses for this node
}

/*
  Collection = GameTree { GameTree }
  GameTree   = "(" Sequence { GameTree } ")"
  Sequence   = Node { Node }
  Node       = ";" { Property }
  Property   = PropIdent PropValue { PropValue }
  PropIdent  = UcLetter { UcLetter }
  PropValue  = "[" CValueType "]"
  CValueType = (ValueType | Compose)
  ValueType  = (None | Number | Real | Double | Color | SimpleText |
		Text | Point  | Move | Stone)

AB: Add Black: locations of Black stones to be placed on the board prior to the first move.
AW: Add White: locations of White stones to be placed on the board prior to the first move.
AN: Annotations: name of the person commenting the game.
AP: Application: application that was used to create the SGF file (e.g. CGOban2,...).
B: a move by Black at the location specified by the property value.
BR: Black Rank: rank of the Black player.
BT: Black Team: name of the Black team.
C: Comment: a comment.
CP: Copyright: copyright information. See Kifu Copyright Discussion.
DT: Date: date of the game.
EV: Event: name of the event (e.g. 58th Honinbo Title Match).
FF: File format: version of SGF specification governing this SGF file.
GM: Game: type of game represented by this SGF file. A property value of 1 refers to Go.
GN: Game Name: name of the game record.
HA: Handicap: the number of handicap stones given to Black. Placement of the handicap stones are set using the AB property.
KM: Komi: komi.
ON: Opening: information about the opening (fuseki), rarely used in any file.
OT: Overtime: overtime system.
PB: Black Name: name of the black player.
PC: Place: place where the game was played (e.g.: Tokyo).
PL: Player: color of player to start.
PW: White Name: name of the white player.
RE: Result: result, usually in the format "B+R" (Black wins by resign) or "B+3.5" (black wins by 3.5 moku).
RO: Round: round (e.g.: 5th game).
RU: Rules: ruleset (e.g.: Japanese).
SO: Source: source of the SGF file.
SZ: Size: size of the board, non square boards are supported.
TM: Time limit: time limit in seconds.
US: User: name of the person who created the SGF file.
W: a move by White at the location specified by the property value.
WR: White Rank: rank of the White player.
WT: White Team: name of the White team.


'list of':    PropValue { PropValue }
'elist of':   ((PropValue { PropValue }) | None)
              In other words elist is list or "[]".
*/
