use std::ops::RangeBounds;

use super::{MapMonoid, Monoid};

pub struct LazySegtree<'a, M: ?Sized + MapMonoid> {
    map_monoid: &'a M,
    lazy: Box<[M::F]>,
    data: Box<[<M::M as Monoid>::S]>,
}

pub struct Range<'a, 'b, M: ?Sized + MapMonoid> {
    slice: &'a mut [<M::M as Monoid>::S],
    map_monoid: &'b M,
}

impl<'a, M: ?Sized + MapMonoid> LazySegtree<'a, M> {
    pub fn new(n: usize) -> Self {
        todo!()
    }

    pub fn range(&mut self, range: impl RangeBounds<usize>) -> Range<'_, 'a, M> {
        todo!()
    }
}