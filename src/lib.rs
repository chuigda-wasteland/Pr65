#![feature(fn_traits)]
#![feature(with_options)]

use std::cmp::Ordering;

mod encode;
mod error;
mod table;
mod partition;
mod io;

pub trait Comparator {
    fn compare(lhs: &[u8], rhs: &[u8]) -> Ordering;
}

pub struct DefaultComparator();

impl Comparator for DefaultComparator {
    fn compare(lhs: &[u8], rhs: &[u8]) -> Ordering {
        lhs.cmp(rhs)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
