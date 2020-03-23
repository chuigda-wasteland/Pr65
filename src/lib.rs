#![feature(fn_traits)]

use std::cmp::Ordering;
use std::marker::PhantomData;

mod encode;
mod error;
mod table;
mod partition;

pub trait Comparator {
    fn compare(lhs: &[u8], rhs: &[u8]) -> Ordering;
}

pub struct DefaultComparator();

impl Comparator for DefaultComparator {
    fn compare(lhs: &[u8], rhs: &[u8]) -> Ordering {
        lhs.cmp(rhs)
    }
}

pub(crate) struct KVPair<Comp: Comparator> {
    key: Vec<u8>,
    value: Vec<u8>,
    phantom: PhantomData<Comp>
}

impl<Comp: Comparator> KVPair<Comp> {
    fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { key, value, phantom: PhantomData }
    }
}

impl<Comp: Comparator> Ord for KVPair<Comp> {
    fn cmp(&self, other: &Self) -> Ordering {
        Comp::compare(&self.key, &other.key)
    }
}

impl<Comp: Comparator> PartialOrd for KVPair<Comp> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Comp: Comparator> PartialEq for KVPair<Comp> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<Comp: Comparator> Eq for KVPair<Comp> {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
