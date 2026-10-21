use super::List;
use super::{Allocator, AllocatorExt};

#[derive(Clone, Copy, Debug)]
pub struct PersistentStack<'a, T: Copy, A: Allocator<'a>> {
    alloc: A,
    unmanaged: Unmanaged<'a, T>,
}

#[derive(Clone, Copy, Debug)]
pub struct Unmanaged<'a, T: Copy>(List<'a, T>);

impl<'a, T: Copy> Default for Unmanaged<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: Copy, A: Allocator<'a> + Clone> PersistentStack<'a, T, A> {
    pub const fn new(alloc: A) -> Self {
        Self {
            alloc,
            unmanaged: Unmanaged::new(),
        }
    }

    pub fn push(&self, value: T) -> Self {
        self.unmanaged
            .push(&self.alloc, value)
            .with_alloc(self.alloc.clone())
    }

    pub fn pop(&self) -> Option<(T, Self)> {
        self.unmanaged
            .pop()
            .map(|(v, s)| (v, s.with_alloc(self.alloc.clone())))
    }

    pub fn peek(&self) -> Option<T> {
        self.unmanaged.peek()
    }

    pub const fn is_empty(&self) -> bool {
        self.unmanaged.is_empty()
    }

    pub fn extract_alloc(self) -> Unmanaged<'a, T> {
        self.unmanaged
    }

    pub fn alloc(&self) -> &A {
        &self.alloc
    }
}

impl<'a, T: Copy> Unmanaged<'a, T> {
    pub const fn new() -> Self {
        Self(List::Nil)
    }

    pub fn push<A: Allocator<'a>>(self, alloc: &A, value: T) -> Self {
        Self(List::Cons(alloc.get_ref((value, self.0))))
    }

    pub fn pop(self) -> Option<(T, Self)> {
        match self.0 {
            List::Nil => None,
            List::Cons(&(h, t)) => Some((h, Self(t))),
        }
    }

    pub fn peek(self) -> Option<T> {
        self.0.as_option().map(|&(h, _)| h)
    }

    pub const fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    pub const fn with_alloc<A: Allocator<'a>>(self, alloc: A) -> PersistentStack<'a, T, A> {
        PersistentStack {
            alloc,
            unmanaged: self,
        }
    }
}

impl<'a, T: Copy> Iterator for Unmanaged<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let (v, new) = self.pop()?;
        *self = new;
        Some(v)
    }
}

impl<'a, T: Copy, A: Allocator<'a>> IntoIterator for PersistentStack<'a, T, A> {
    type IntoIter = Unmanaged<'a, T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        self.unmanaged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::ThreadMemPool;
    use rand::RngExt;

    #[test]
    fn test() {
        let pool = ThreadMemPool::new();
        let mut expected = Vec::new();
        let mut stacks = Vec::new();
        let mut rng = rand::rng();
        stacks.push(PersistentStack::new(pool.allocator()));
        expected.push(vec![]);
        for i in 0..100 {
            let j = rng.random_range(0..=i);
            stacks.push(stacks[j].push(i));
            expected.push(
                expected[j]
                    .iter()
                    .copied()
                    .chain(std::iter::once(i))
                    .collect(),
            );
        }

        for (expected, &stack) in expected.iter().zip(&stacks) {
            assert!(expected.iter().copied().rev().eq(stack));
        }
    }
}
