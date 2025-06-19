#![feature(allocator_api, box_vec_non_null, maybe_uninit_slice, ptr_as_ref_unchecked)]
// #![no_std]
extern crate alloc;
use alloc::{alloc::{Allocator, Global}, boxed::Box};
use core::{cell::Cell, mem::MaybeUninit, pin::Pin, ptr::NonNull};

pub struct Arena<T, const N: usize, A: Allocator + Clone = Global> {
    root: Cell<Option<NonNull<Chunk<T, N>>>>,
    alloc: A,
}

impl<T, const N: usize> Arena<T, N, Global> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self::new_in(Global)
    }
}

impl<T, const N: usize, A: Allocator + Clone> Arena<T, N, A> {
    #[inline(always)]
    pub const fn new_in(alloc: A) -> Self {
        assert!(N > 0);
        Self { root: Cell::new(None), alloc }
    }

    #[inline]
    pub fn alloc(&self, value: T) -> &mut T {
        if let Some(chunk) = self.root.get().map(|mut chunk| unsafe { chunk.as_mut() }) {
            if chunk.len < N {
                // SAFETY: `slice::get_unchecked_mut` is never called if `chunk.len >= N`.
                let value = unsafe { chunk.data.get_unchecked_mut(chunk.len).write(value) as *mut T };
                chunk.len += 1;
                // SAFETY: `value` is never null.
                return unsafe { value.as_mut_unchecked() }
            }
        }
        let mut chunk = Chunk {
            prev: self.root.get(),
            len: 1,
            data: [const { MaybeUninit::uninit() }; N],
        };
        // SAFETY: `Self` cannot be constructed without specifying a nonzero value for `N`.
        unsafe { chunk.data.get_unchecked_mut(0).write(value); }
        self.root.set(Some(Box::into_non_null(Box::new_in(chunk, self.alloc.clone()))));
        // SAFETY: `self.root` was just set to a non-null `Some` value.
        // SAFETY: `Self` cannot be constructed without specifying a nonzero value for `N`.
        // SAFETY: `self.root`'s first element was just initialized.
        unsafe { self.root.get().unwrap_unchecked().as_mut().data.get_unchecked_mut(0).assume_init_mut() }
    }

    #[inline(always)]
    pub fn alloc_pinned(&self, value: T) -> Pin<&mut T> {
        // SAFETY: Values allocated via this arena are never moved.
        unsafe { Pin::new_unchecked(self.alloc(value)) }
    }
}

impl<T, const N: usize, A: Allocator + Clone> Drop for Arena<T, N, A> {
    #[inline]
    fn drop(&mut self) {
        let mut curr = self.root.get();
        while let Some(chunk) = curr {
            // SAFETY: `Box::from_non_null_in` is only called once on `chunk`.
            let mut boxed = unsafe { Box::from_non_null_in(chunk, self.alloc.clone()) };
            // SAFETY: `boxed.len` is never greater than N.
            let (init, _) = unsafe { boxed.data.split_at_mut_unchecked(boxed.len) };
            // SAFETY: All indices of `boxed.data` from `[0, boxed.len)` are initialized.
            unsafe { init.assume_init_drop() };
            curr = boxed.prev;
        }
    }
}

struct Chunk<T, const N: usize> {
    prev: Option<NonNull<Self>>,
    len: usize,
    data: [MaybeUninit<T>; N],
}

#[cfg(test)]
mod tests {
    use crate::Arena;

    #[test]
    fn it_works() {
        let arena = Arena::<i32, 8>::new();
        for i in 0..100 {
            assert_eq!(*arena.alloc(i), i);
        }
    }
}

