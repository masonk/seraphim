use std::fmt;
use std::collections::HashMap;
use regex;
#[derive(Clone, PartialEq, Eq, Hash)]
// Displayed as cR, e.g., a 13,
// but in code designated as row, col
// rows and cols are INDEXED FROM ZERO
pub struct Pos(pub i8, pub i8);

impl Pos {
    pub fn parse(s: &str) -> Self {
        lazy_static! {
            static ref RE : regex::Regex = regex::Regex::new(r"([a-s])\s*(\d+)").unwrap();
            static ref CHARS: Vec<char> = {
                "abcdefghijklmnopqrs".chars().collect::<Vec<char>>()
              // 123456789
            };
            static ref COLMAP: HashMap<char, i8> = {
                let mut map = HashMap::new();
                let pairs = (0..19).zip(CHARS.iter());
                for (i, c) in pairs {
                    map.insert(c.clone(), i);
                }
                map
            };
        }

        let cap = RE.captures(s).unwrap();
        let colchar = cap[1].chars().next().unwrap();

        let rowidx = i8::from_str_radix(&cap[2], 10).unwrap() - 1;
        let colidx = COLMAP[&colchar];
        Pos(rowidx, colidx)
    }

    // the cardinal neighbors of self
    pub fn neighbors(&self) -> impl ExactSizeIterator<Item = Pos> {
        let &Pos(ref i, ref j) = self;
        let mut vec = vec![];

        for o in [-1, 1].iter() {
            let it = i + *o;
            let jt = j + *o;

            if it >= 0 && it < 19 {
                vec.push(Pos(it, *j));
            }
            if jt >= 0 && jt < 19 {
                vec.push(Pos(*i, jt));
            }
        }
        vec.into_iter()
    }
}
impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lazy_static! {
            static ref CHARS: Vec<char> = {
                "abcdefghijklmnopqrs".chars().collect::<Vec<char>>()
            };
        }
        let &Pos(row, col) = self;
        let c = CHARS[col as usize];
        write!(f, "{} {}", c, row + 1)
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::Pos;
    #[test]
    fn display() {
        let actual = format!("{}", Pos(13, 12));
        assert_eq!(actual, "m 14");

        let a2 = format!("{}", Pos(1, 1));
        assert_eq!(a2, "b 2");

        let a3 = format!("{}", Pos(0, 0));
        assert_eq!(a3, "a 1");

        let a4 = format!("{}", Pos(18, 18));
        assert_eq!(a4, "s 19");

        let a5 = format!("{}", Pos(0, 18));
        assert_eq!(a5, "s 1");
    }
    #[test]
    fn display_roundtrips() {
        let expected = Pos(13, 12);
        let format = format!("{}", expected);
        let parse = Pos::parse(&format);
        assert_eq!(parse, expected);
    }

    #[test]
    fn a_parse() {
        let actual = Pos::parse("e 9");
        assert_eq!(actual, Pos(8, 4));
    }

    fn neighbors_exactly(pos: Pos, expected: Vec<Pos>) {
        assert_eq!(pos.neighbors().into_iter().len(), expected.len());
        for e in &expected {
            assert!(
                pos.neighbors().into_iter().find(|a| a == e).is_some(),
                "{:?} not found in {:?}",
                e,
                pos.neighbors().collect::<Vec<Pos>>()
            );
        }
    }
    #[test]
    fn a1_neighbors() {
        // hah hah, test the corner cases
        let bl = Pos::parse("a 1");
        let bl_expected = vec![Pos::parse("a 2"), Pos::parse("b 1")];
        neighbors_exactly(bl, bl_expected);
    }
    #[test]
    fn a19_neighbors() {
        let tl = Pos::parse("a 19");
        let tl_expected = vec![Pos::parse("b 19"), Pos::parse("a 18")];
        neighbors_exactly(tl, tl_expected);
    }
    #[test]
    fn s1_neighbors() {
        let br = Pos::parse("s 1");
        let br_expected = vec![Pos::parse("s 2"), Pos::parse("r 1")];
        neighbors_exactly(br, br_expected);
    }
    #[test]
    fn s19_neighbors() {
        let tr = Pos::parse("s 19");
        let tr_expected = vec![Pos::parse("r 19"), Pos::parse("s 18")];
        neighbors_exactly(tr, tr_expected);
    }
    #[test]
    fn f15_neighbors() {
        let mid = Pos::parse("f 15");
        let mid_expected = vec![
            Pos::parse("e 15"),
            Pos::parse("g 15"),
            Pos::parse("f 14"),
            Pos::parse("f 16"),
        ];
        neighbors_exactly(mid, mid_expected);
    }
}
