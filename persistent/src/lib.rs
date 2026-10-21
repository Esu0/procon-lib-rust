pub mod memory;
pub mod stack;
pub mod queue;

use core::ptr::NonNull;

pub trait Allocator<'a> {
    ///
    /// 型Tの値1個分のメモリを確保する。領域は初期化されている必要はない。
    /// メモリ確保に失敗した場合は[`Option::None`]を返す。
    ///
    /// # Safety
    ///
    /// - 返すポインタはユニークであり、ポインタが指す領域は他の確保済みの領域と重複してはならない
    /// - 返すポインタが指す領域は'aの期間有効でなければならない
    /// - 返すポインタが指す領域は'aの期間借用されるため、排他参照をとってはいけない
    unsafe fn alloc<T: 'a>(&self) -> Option<NonNull<T>>;

    fn alloc_imm<T>(&self, value: T) -> Option<&'a T> {
        unsafe {
            let ptr = self.alloc::<T>()?;
            ptr.write(value);
            Some(&*ptr.as_ptr())
        }
    }
}

trait AllocatorExt<'a>: Allocator<'a> {
    fn get_ref<T>(&self, value: T) -> &'a T {
        self.alloc_imm(value).unwrap_or_else(|| panic!("allocation failed"))
    }
}

impl<'a, A: Allocator<'a>> AllocatorExt<'a> for A {}

#[derive(Debug)]
enum List<'a, T>{
    Nil,
    Cons(&'a (T, List<'a, T>)),
}

impl<'a, T> Clone for List<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for List<'a, T> {}

impl<'a, T: Clone> List<'a, T> {
    fn rev<A: Allocator<'a>>(self, alloc: &A) -> Self {
        let mut list = Self::Nil;
        let mut cur = self;
        while let Self::Cons((h, t)) = cur {
            list = Self::Cons(alloc.get_ref((h.clone(), list)));
            cur = *t;
        }
        list
    }

    fn concat<A: Allocator<'a>>(self, alloc: &A, other: Self) -> Self {
        use List::*;
        // TODO: no recursion implementation
        match self {
            Nil => other,
            Cons((h, t)) => Cons(alloc.get_ref((h.clone(), t.concat(alloc, other))))
        }
    }

    const fn as_option(self) -> Option<&'a (T, List<'a, T>)> {
        match self {
            Self::Nil => None,
            Self::Cons(ht) => Some(ht),
        }
    }

    const fn is_empty(self) -> bool {
        matches!(self, Self::Cons(_))
    }
}
