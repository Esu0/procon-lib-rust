use std::marker::PhantomData;

use super::Allocator;

#[derive(Debug)]
struct Node<'a, T: Copy> {
    head: T,
    tail: Option<&'a Node<'a, T>>,
}

#[derive(Clone, Copy, Debug)]
pub struct PersistentStack<'a, T: Copy, A: Allocator<'a>> {
    alloc: A,
    unmanaged: Unmanaged<'a, T>,
}

#[derive(Clone, Copy, Debug)]
pub struct Unmanaged<'a, T: Copy> {
    node_ref: Option<&'a Node<'a, T>>,
    _marker: PhantomData<fn (&'a ()) -> &'a ()>,
}

impl<'a, T: Copy> Clone for Node<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: Copy> Copy for Node<'a, T> {}

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
        self.unmanaged.push(&self.alloc, value).with_alloc(self.alloc.clone())
    }

    pub fn pop(&self) -> Option<(T, Self)> {
        self.unmanaged.pop().map(|(v, s)| (v, s.with_alloc(self.alloc.clone())))
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

fn new_node<'a, T: Copy, A: Allocator<'a>>(alloc: &A, head: T, tail: Option<&'a Node<'a, T>>) -> &'a Node<'a, T> {
    alloc.alloc_imm(Node { head, tail }).unwrap_or_else(|| panic!("allocation failed"))
}

impl<'a, T: Copy> Unmanaged<'a, T> {
    pub const fn new() -> Self {
        Self::from_node_ref(None)
    }

    const fn from_node_ref(node_ref: Option<&'a Node<'a, T>>) -> Self {
        Self {
            node_ref,
            _marker: PhantomData,
        }
    }

    pub fn push<A: Allocator<'a>>(self, alloc: &A, value: T) -> Self {
        Self::from_node_ref(Some(new_node(alloc, value, self.node_ref)))
    }

    pub fn pop(self) -> Option<(T, Self)> {
        let node = self.node_ref?;
        Some((node.head, Self::from_node_ref(node.tail)))
    }

    pub fn peek(self) -> Option<T> {
        self.node_ref.map(|n| n.head)
    }

    pub const fn is_empty(self) -> bool {
        self.node_ref.is_some()
    }

    pub const fn with_alloc<A: Allocator<'a>>(self, alloc: A) -> PersistentStack<'a, T, A> {
        PersistentStack { alloc, unmanaged: self }
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
            expected.push(expected[j].iter().copied().chain(std::iter::once(i)).collect());
        }

        for (expected, &stack) in expected.iter().zip(&stacks) {
            assert!(expected.iter().copied().rev().eq(stack));
        }
    }
}
