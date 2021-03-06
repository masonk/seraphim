use std::fmt;
use std::collections::HashMap;
use regex;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// Displayed as cR, e.g., a 13,
// but in code designated as row, col
// rows and cols are INDEXED FROM ZERO
pub struct Pos19(pub usize);

impl Pos19 {
    pub fn from_coords(c: usize, r: usize) -> Self {
        Pos19(c + r * 19)
    }
    pub fn from_sgf_coords(c: char, r: char) -> Self {
        lazy_static! {
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
        Pos19::from_coords(COLMAP[&c] as usize, COLMAP[&r] as usize)
    }
    pub fn to_coords(&self) -> (usize, usize) {
        // i = c + r * 19
        // (i - c)/19 = r
        // i - (r * 19) = c
        let &Pos19(i) = self;
        let r = i / 19;
        let c = i - (r * 19);
        (c, r)
    }
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
        Pos19::from_coords(colidx as usize, rowidx as usize)
    }

    // the cardinal neighbors of self
    pub fn neighbors(&self) -> impl ExactSizeIterator<Item = Pos19> {
        let (j, i) = self.to_coords();
        let mut vec = vec![];

        for o in [-1isize, 1].iter() {
            let it = ((i as isize) + *o) as isize;
            let jt = ((j as isize) + *o) as isize;

            if it >= 0 && it < 19 {
                vec.push(Pos19::from_coords(j, it as usize));
            }
            if jt >= 0 && jt < 19 {
                vec.push(Pos19::from_coords(jt as usize, i));
            }
        }
        vec.into_iter()
    }
    pub fn pretty(&self) -> String {
        format!("{}", self)
    }
}
impl fmt::Debug for Pos19 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (Pos({})", self, self.0)
    }
}

impl fmt::Display for Pos19 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lazy_static! {
            static ref CHARS: Vec<char> = {
                "abcdefghijklmnopqrs".chars().collect::<Vec<char>>()
            };
        }
        let (col, row) = self.to_coords();
        let c = CHARS[col];
        write!(f, "{} {}", c, row + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::Pos19;
    #[test]
    fn display() {
        let a2 = format!("{}", Pos19::from_coords(0, 1));
        assert_eq!(a2, "a 2");

        let b2 = format!("{}", Pos19::from_coords(1, 1));
        assert_eq!(b2, "b 2");

        let m14 = format!("{}", Pos19::from_coords(12, 13));
        assert_eq!(m14, "m 14");

        let a3 = format!("{}", Pos19::from_coords(0, 0));
        assert_eq!(a3, "a 1");

        let a4 = format!("{}", Pos19::from_coords(18, 18));
        assert_eq!(a4, "s 19");

        let a5 = format!("{}", Pos19::from_coords(18, 0));
        assert_eq!(a5, "s 1");
    }
    #[test]
    fn display_roundtrips() {
        let expected = Pos19::from_coords(12, 13);
        let format = format!("{}", expected);
        let parse = Pos19::parse(&format);
        assert_eq!(parse, expected);
    }

    #[test]
    fn a_parse() {
        let actual = Pos19::parse("e 9");
        assert_eq!(actual, Pos19::from_coords(4, 8));
    }

    fn neighbors_exactly(pos: Pos19, expected: Vec<Pos19>) {
        assert_eq!(pos.neighbors().into_iter().len(), expected.len());
        for e in &expected {
            assert!(
                pos.neighbors().into_iter().find(|a| a == e).is_some(),
                "{:?} not found in {:?}",
                e,
                pos.neighbors().collect::<Vec<Pos19>>()
            );
        }
    }
    #[test]
    fn a1_neighbors() {
        // hah hah, test the corner cases
        let bl = Pos19::parse("a 1");
        let bl_expected = vec![Pos19::parse("a 2"), Pos19::parse("b 1")];
        neighbors_exactly(bl, bl_expected);
    }
    #[test]
    fn a19_neighbors() {
        let tl = Pos19::parse("a 19");
        let tl_expected = vec![Pos19::parse("b 19"), Pos19::parse("a 18")];
        neighbors_exactly(tl, tl_expected);
    }
    #[test]
    fn s1_neighbors() {
        let br = Pos19::parse("s 1");
        let br_expected = vec![Pos19::parse("s 2"), Pos19::parse("r 1")];
        neighbors_exactly(br, br_expected);
    }
    #[test]
    fn s19_neighbors() {
        let tr = Pos19::parse("s 19");
        let tr_expected = vec![Pos19::parse("r 19"), Pos19::parse("s 18")];
        neighbors_exactly(tr, tr_expected);
    }
    #[test]
    fn f15_neighbors() {
        let mid = Pos19::parse("f 15");
        let mid_expected = vec![
            Pos19::parse("e 15"),
            Pos19::parse("g 15"),
            Pos19::parse("f 14"),
            Pos19::parse("f 16"),
        ];
        neighbors_exactly(mid, mid_expected);
    }
}
