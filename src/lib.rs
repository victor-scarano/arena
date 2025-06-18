#![feature(box_vec_non_null, maybe_uninit_slice, ptr_as_ref_unchecked)]
#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use core::{cell::Cell, mem::MaybeUninit, ptr::NonNull};

struct Block<T, const N: usize> {
    prev: Option<NonNull<Self>>,
    len: usize,
    data: [MaybeUninit<T>; N],
}

pub struct Arena<T, const N: usize> {
    root: Cell<Option<NonNull<Block<T, N>>>>,
}

impl<T, const N: usize> Arena<T, N> {
    pub const fn new() -> Self {
        assert!(N > 0);
        Self { root: Cell::new(None) }
    }

    pub fn alloc(&self, value: T) -> &mut T {
        if let Some(block) = self.root.get().map(|mut block| unsafe { block.as_mut() }) {
            if block.len < N {
                let value = unsafe { block.data.get_unchecked_mut(block.len).write(value) as *mut T };
                block.len += 1;
                return unsafe { value.as_mut_unchecked() }
            }
        }
        let mut block = Block {
            prev: self.root.get(),
            len: 1,
            data: [const { MaybeUninit::uninit() }; N],
        };
        unsafe { block.data.get_unchecked_mut(0).write(value); }
        self.root.set(Some(Box::into_non_null(Box::new(block))));
        unsafe { self.root.get().unwrap_unchecked().as_mut().data.get_unchecked_mut(0).assume_init_mut() }
    }
}

impl<T, const N: usize> Drop for Arena<T, N> {
    fn drop(&mut self) {
        let mut curr = self.root.get();
        while let Some(block) = curr {
            let mut boxed = unsafe { Box::from_non_null(block) };
            let (init, _) = unsafe { boxed.data.split_at_mut_unchecked(boxed.len) };
            unsafe { init.assume_init_drop() };
            curr = boxed.prev;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Arena;

    #[test]
    fn it_works() {
        let arena = Arena::<i32, 8>::new();
        for i in 0..10 {
            assert_eq!(*arena.alloc(i), i);
        }
    }
}
