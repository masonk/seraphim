use std::fmt;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq)]
// Displayed as cR, e.g., a13,
// but in code designated as row, col
pub struct Pos<'a, 'b>(pub &'a u8, pub &'b u8);
lazy_static! {
    static ref COLMAP: HashMap<u8, char> = {
        let mut map = HashMap::new();
        let pairs = (1..20).zip("abcdefghijklmnopqrs".chars());
        for (i, c) in pairs {
            map.insert(i, c);
        }
        map
    };
}
impl<'a, 'b> fmt::Debug for Pos<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Pos(ref row, ref col) = self;
        let c = COLMAP.get(col).unwrap();
        write!(f, "{}{}", c, row)
    }
}

impl<'a, 'b> fmt::Display for Pos<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
