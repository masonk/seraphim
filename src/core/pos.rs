use std::fmt;
use std::collections::HashMap;
use regex;

#[derive(Clone, PartialEq, Eq)]
// Displayed as cR, e.g., a 13,
// but in code designated as row, col
// rows and cols are INDEXED FROM ZERO
pub struct Pos(pub u8, pub u8);

impl Pos {
    pub fn parse(s: &str) -> Self {
        lazy_static! {
            static ref RE : regex::Regex = regex::Regex::new(r"([a-s])\s*(\d+)").unwrap();
            static ref CHARS: Vec<char> = {
                "abcdefghijklmnopqrs".chars().collect::<Vec<char>>()
              // 123456789
            };
            static ref COLMAP: HashMap<char, u8> = {
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

        let rowidx = u8::from_str_radix(&cap[2], 10).unwrap() - 1;
        let colidx = COLMAP[&colchar];
        Pos(rowidx, colidx)
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
}
