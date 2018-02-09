#![feature(conservative_impl_trait, universal_impl_trait)]

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
