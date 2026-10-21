use std::{alloc::Layout, cell::Cell, ptr::NonNull};
use super::Allocator;
use std::alloc;

pub struct ThreadMemPool {
    list: Cell<Option<NonNull<Node>>>,
}

#[derive(Clone, Copy)]
struct Node {
    layout: Layout,
    ptr: *mut u8,
    next: Option<NonNull<Node>>,
}

#[derive(Clone, Copy)]
pub struct ThreadMemPoolAlloc<'a>(&'a ThreadMemPool);

impl Drop for ThreadMemPool {
    fn drop(&mut self) {
        let node_layout = Layout::new::<Node>();
        let mut node = self.list.get();
        while let Some(p) = node {
            unsafe {
                let Node { layout, ptr, next } = *p.as_ref();
                node = next;
                alloc::dealloc(ptr, layout);
                alloc::dealloc(p.as_ptr().cast(), node_layout);
            }
        }
    }
}

impl Default for ThreadMemPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadMemPool {
    pub const fn new() -> Self {
        Self {
            list: Cell::new(None),
        }
    }

    pub fn allocator(&self) -> ThreadMemPoolAlloc<'_> {
        ThreadMemPoolAlloc(self)
    }
}

impl<'a> Allocator<'a> for ThreadMemPoolAlloc<'a> {
    unsafe fn alloc<T: 'a>(&self) -> Option<NonNull<T>> {
        if size_of::<T>() == 0 {
            return Some(NonNull::dangling());
        }
        let layout = Layout::new::<T>();
        let ptr = NonNull::new(unsafe { alloc::alloc(layout) })?;
        let new_node = NonNull::new(unsafe { alloc::alloc(Layout::new::<Node>()) })?.cast::<Node>();
        let old_list = self.0.list.get();
        unsafe {
            new_node.write(Node {
                layout,
                ptr: ptr.as_ptr(),
                next: old_list,
            });
        }
        self.0.list.set(Some(new_node));
        Some(ptr.cast())
    }
}
