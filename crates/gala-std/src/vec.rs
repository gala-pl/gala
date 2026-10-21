use core::ops::Index;
use core::ptr;
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};

pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VecError {
    CapacityOverflow,
    LayoutError,
    AllocationFailed,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Vec { ptr: ptr::null_mut(), len: 0, cap: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap && self.try_reserve(1).is_err() {
            handle_alloc_error(Layout::new::<T>());
        }
        // SAFETY: the capacity check above guarantees one initialized slot is
        // available and the slot has not previously been initialized.
        unsafe { ptr::write(self.ptr.add(self.len), value) };
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: the decremented length identifies the last initialized slot.
        Some(unsafe { ptr::read(self.ptr.add(self.len)) })
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            // SAFETY: the bounds check guarantees an initialized element.
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), VecError> {
        let required = self.len.checked_add(additional).ok_or(VecError::CapacityOverflow)?;
        if required <= self.cap {
            return Ok(());
        }
        if core::mem::size_of::<T>() == 0 {
            self.ptr = ptr::NonNull::<T>::dangling().as_ptr();
            self.cap = usize::MAX;
            return Ok(());
        }
        let mut new_cap = self.cap.max(4);
        while new_cap < required {
            new_cap = new_cap.checked_mul(2).ok_or(VecError::CapacityOverflow)?;
        }
        let layout = Layout::array::<T>(new_cap).map_err(|_| VecError::LayoutError)?;
        // SAFETY: `layout` was constructed for T and new_cap elements.
        let new_ptr = unsafe { alloc(layout) as *mut T };
        if new_ptr.is_null() {
            return Err(VecError::AllocationFailed);
        }
        if self.len != 0 {
            // SAFETY: both allocations are valid for `len` T values and do
            // not overlap because `new_ptr` is freshly allocated.
            unsafe { ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len) };
        }
        if self.cap != 0 {
            let old_layout = Layout::array::<T>(self.cap).map_err(|_| VecError::LayoutError)?;
            // SAFETY: old_layout describes the allocation held by self.ptr.
            unsafe { dealloc(self.ptr as *mut u8, old_layout) };
        }
        self.ptr = new_ptr;
        self.cap = new_cap;
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        self.clear();
        if self.cap != 0 && core::mem::size_of::<T>() != 0 {
            if let Ok(layout) = Layout::array::<T>(self.cap) {
                // SAFETY: layout describes the allocation created in
                // try_reserve and all elements have already been dropped.
                unsafe { dealloc(self.ptr as *mut u8, layout) };
            }
        }
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.get(index).expect("index out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vec_is_empty() {
        let v: Vec<i32> = Vec::new();
        assert!(v.is_empty());
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_default_vec_is_empty() {
        let v: Vec<i32> = Vec::default();
        assert!(v.is_empty());
    }

    #[test]
    fn test_get_out_of_bounds() {
        let v: Vec<i32> = Vec::new();
        assert_eq!(v.get(0), None);
        assert_eq!(v.get(100), None);
    }

    #[test]
    fn test_push_pop_and_growth() {
        let mut v = Vec::new();
        for value in 0..10 {
            v.push(value);
        }
        assert_eq!(v.len(), 10);
        assert!(v.capacity() >= 10);
        assert_eq!(v.get(4), Some(&4));
        assert_eq!(v.pop(), Some(9));
        assert_eq!(v.len(), 9);
    }

    #[test]
    fn test_clear_drops_elements() {
        let mut v = Vec::new();
        v.push(String::from("gala"));
        v.clear();
        assert!(v.is_empty());
    }
}
