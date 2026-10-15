use std::ops::RangeBounds;

use super::{MapMonoid, Monoid};

pub struct LazySegtree<M: MapMonoid> {
    map_monoid: M,
    lazy: Box<[M::F]>,
    data: Box<[<M::M as Monoid>::S]>,
}

pub struct Range<'a, M: MapMonoid> {
    lazy_segtree: &'a mut LazySegtree<M>,
}

impl<M: MapMonoid> LazySegtree<M> {
    pub fn new(map_monoid: M, n: usize) -> Self {
        todo!()
    }

    pub fn range(&mut self, range: impl RangeBounds<usize>) -> Range<'_, M> {
        todo!()
    }
}
