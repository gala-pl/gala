use core::ops::Index;
use core::ptr;

pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    #[allow(dead_code)]
    cap: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Vec {
            ptr: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, _value: T) {
        unimplemented!("Vec::push requires an allocator")
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
}