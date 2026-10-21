
use crate::{List, Allocator, AllocatorExt, lazy::{Susp, InitFn}};

struct ConcatRev<'a, T>(List<'a, T>, List<'a, T>);

impl<'a, T: Clone, A: Allocator<'a>> InitFn<A> for ConcatRev<'a, T> {
    type Output = List<'a, T>;
    fn eval(self, alloc: A) -> Self::Output {
        let Self(front, rear) = self;
        front.concat(&alloc, rear.rev(&alloc))
    }
}

pub struct Unmanaged<'a, T, A: Allocator<'a>> {
    working_copy: List<'a, T>,
    front: &'a Susp<List<'a, T>, ConcatRev<'a, T>, A>,
    rear: List<'a, T>,
    front_length: usize,
    rear_length: usize,
}

pub struct PersistentQueue<'a, T, A: Allocator<'a>> {
    unmanaged: Unmanaged<'a, T, A>,
    alloc: A,
}

impl<'a, T: Clone, A: Allocator<'a> + Clone> Unmanaged<'a, T, A> {
    pub fn new(alloc: A) -> Self {
        Self {
            working_copy: List::Nil,
            front: alloc.get_ref(Susp::init(List::Nil)),
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

    pub fn push(self, alloc: A, value: T) -> Self {

    }
}