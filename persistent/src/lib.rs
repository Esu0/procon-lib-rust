pub mod memory;
pub mod stack;

use core::{cell::Cell, ptr::NonNull};

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

    fn alloc_inner_mut<T>(&self, value: T) -> Option<&'a Cell<T>> {
        unsafe {
            let ptr = self.alloc::<T>()?;
            ptr.write(value);
            Some(&*ptr.cast::<Cell<T>>().as_ptr())
        }
    }
}
