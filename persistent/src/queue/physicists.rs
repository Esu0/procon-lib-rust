use core::fmt;

use crate::{
    Allocator, AllocatorExt, List,
    lazy::{InitFn, Susp},
};

#[derive(Clone, Debug)]
enum InitFns<'a, T, A> {
    ConcatRev(List<'a, T>, &'a (T, List<'a, T>)),
    TailForce(&'a ListSusp<'a, T, A>),
}

impl<'a, T: Copy, A: Allocator<'a>> InitFn<A> for InitFns<'a, T, A> {
    type Output = &'a (T, List<'a, T>);
    fn eval(self, alloc: A) -> Self::Output {
        let List::Cons(out) = (match self {
            Self::ConcatRev(front, rear) => front.concat(&alloc, List::Cons(rear).rev(&alloc)),
            Self::TailForce(f) => f.force(alloc).1,
        }) else {
            unreachable!();
        };
        out
    }
}

type ListSusp<'a, T, A> = Susp<&'a (T, List<'a, T>), InitFns<'a, T, A>, A>;

pub struct Unmanaged<'a, T, A: Allocator<'a>> {
    working_copy: List<'a, T>,
    front: Option<&'a ListSusp<'a, T, A>>,
    rear: List<'a, T>,
    front_length: usize,
    rear_length: usize,
}

impl<'a, T: Clone, A: Allocator<'a>> Clone for Unmanaged<'a, T, A> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<'a, T: Copy, A: Allocator<'a>> Copy for Unmanaged<'a, T, A> {}

impl<'a, T: fmt::Debug, A: Allocator<'a>> fmt::Debug for Unmanaged<'a, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unmanaged")
            .field("working_copy", &self.working_copy)
            .field("front", &self.front)
            .field("rear", &self.rear)
            .field("front_length", &self.front_length)
            .field("rear_length", &self.rear_length)
            .finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PersistentQueue<'a, T, A: Allocator<'a>> {
    unmanaged: Unmanaged<'a, T, A>,
    alloc: A,
}

impl<'a, T: Copy, A: Allocator<'a> + Copy> Default for Unmanaged<'a, T, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: Copy, A: Allocator<'a> + Copy> Unmanaged<'a, T, A> {
    pub const fn new() -> Self {
        Self {
            working_copy: List::Nil,
            front: None,
            front_length: 0,
            rear: List::Nil,
            rear_length: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.front_length == 0
    }

    pub fn len(&self) -> usize {
        self.front_length + self.rear_length
    }

    pub fn push(&self, alloc: A, value: T) -> Self {
        let mut new = *self;
        new.rear = List::Cons(alloc.get_ref((value, self.rear)));
        new.rear_length += 1;
        new.check_rw(alloc);
        new
    }

    pub fn pop(&self, alloc: A) -> Option<(T, Self)> {
        let mut new = *self;
        let &(h, t) = new.working_copy.as_option()?;
        assert!(new.front_length > 0);
        new.front_length -= 1;
        new.working_copy = t;
        if new.front_length == 0 {
            new.front = None;
        } else {
            let Some(f) = new.front else { unreachable!() };
            new.front = Some(alloc.get_ref(ListSusp::new(InitFns::TailForce(f))));
        }
        new.check_rw(alloc);
        Some((h, new))
    }

    pub fn peek(&self) -> Option<T> {
        self.working_copy.as_option().map(|&(h, _)| h)
    }

    pub fn with_alloc(self, alloc: A) -> PersistentQueue<'a, T, A> {
        PersistentQueue { unmanaged: self, alloc }
    }

    fn check_rw(&mut self, alloc: A) {
        if self.rear_length > self.front_length {
            self.working_copy = match self.front {
                None => List::Nil,
                Some(f) => List::Cons(f.force(alloc)),
            };
            self.front_length += self.rear_length;
            self.rear_length = 0;
            let List::Cons(r) = self.rear else {
                unreachable!()
            };
            self.front =
                Some(alloc.get_ref(ListSusp::new(InitFns::ConcatRev(self.working_copy, r))));
            self.rear = List::Nil;
        }
        if let List::Nil = self.working_copy {
            self.working_copy = match self.front {
                None => List::Nil,
                Some(f) => List::Cons(f.force(alloc)),
            };
        }
    }
}

impl<'a, T: Copy, A: Allocator<'a> + Copy> PersistentQueue<'a, T, A> {
    pub const fn new(alloc: A) -> Self {
        Self {
            unmanaged: Unmanaged::new(),
            alloc,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.unmanaged.is_empty()
    }

    pub fn len(&self) -> usize {
        self.unmanaged.len()
    }

    pub fn push(&self, value: T) -> Self {
        Self {
            unmanaged: self.unmanaged.push(self.alloc, value),
            alloc: self.alloc,
        }
    }

    pub fn pop(&self) -> Option<(T, Self)> {
        self.unmanaged.pop(self.alloc).map(|(v, s)| {
            (
                v,
                Self {
                    unmanaged: s,
                    alloc: self.alloc,
                },
            )
        })
    }

    pub fn extract_alloc(self) -> Unmanaged<'a, T, A> {
        self.unmanaged
    }
}

#[cfg(test)]
mod tests {
    use rand::RngExt;
    use super::*;
    use crate::memory::ThreadMemPool;

    #[test]
    fn test() {
        let pool = ThreadMemPool::new();
        let mut expected = Vec::new();
        let mut queues = Vec::new();
        let mut rng = rand::rng();
        queues.push(PersistentQueue::new(pool.allocator()));
        expected.push(Vec::new());
        for i in 0..100 {
            let j = rng.random_range(0..=i);
            queues.push(queues[j].push(i));
            expected.push(
                std::iter::once(i).chain(expected[j].iter().copied()).collect()
            );
        }

        for (expected, queue) in expected.iter().zip(&queues) {
            let mut q = *queue;
            let mut expected = expected.clone();
            while let Some((v, nq)) = q.pop() {
                q = nq;
                assert_eq!(expected.pop(), Some(v));
            }
        }
    }
}
