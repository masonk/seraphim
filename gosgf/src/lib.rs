#[macro_use]
extern crate lazy_static;
extern crate regex;

pub mod gosgf;
pub use self::gosgf::*;
pub mod parse_sgf;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
