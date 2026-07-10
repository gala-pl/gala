//! Region-based memory allocation for the Gala runtime.
//!
//! Regions provide bump-allocated arenas for short-lived allocations
//! that are freed all at once.

#![allow(clippy::mut_from_ref)]

use std::cell::RefCell;

/// A region allocator that hands out references with the region's lifetime.
pub struct Region {
    pages: RefCell<Vec<Page>>,
    page_size: usize,
}

struct Page {
    data: Vec<u8>,
    cursor: usize,
}

impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}

impl Region {
    /// Creates a new region with the default page size (64 KB).
    pub fn new() -> Self {
        Region::with_page_size(64 * 1024)
    }

    /// Creates a new region with a specific page size.
    pub fn with_page_size(page_size: usize) -> Self {
        Region {
            pages: RefCell::new(Vec::new()),
            page_size,
        }
    }

    /// Allocates space for a value of type `T`.
    pub fn alloc<T>(&self, val: T) -> &mut T {
        let layout = std::alloc::Layout::new::<T>();
        let (ptr, _) = self.alloc_raw(layout);
        let ptr = ptr as *mut T;
        unsafe {
            ptr.write(val);
            &mut *ptr
        }
    }

    /// Allocates a slice of `n` values of type `T`.
    pub fn alloc_slice<T: Copy>(&self, vals: &[T]) -> &mut [T] {
        let layout = std::alloc::Layout::array::<T>(vals.len()).unwrap();
        let (ptr, _) = self.alloc_raw(layout);
        let ptr = ptr as *mut T;
        unsafe {
            std::ptr::copy_nonoverlapping(vals.as_ptr(), ptr, vals.len());
            std::slice::from_raw_parts_mut(ptr, vals.len())
        }
    }

    fn alloc_raw(&self, layout: std::alloc::Layout) -> (*mut u8, usize) {
        let size = layout.size();
        let align = layout.align();

        let mut pages = self.pages.borrow_mut();

        if let Some(page) = pages.last_mut() {
            let aligned_offset = align_up(page.cursor, align);
            if aligned_offset + size <= page.data.len() {
                page.cursor = aligned_offset + size;
                return (unsafe { page.data.as_mut_ptr().add(aligned_offset) }, size);
            }
        }

        let page_size = self.page_size.max(size + align);
        let mut page = Page::new(page_size);
        let aligned_offset = align_up(0, align);
        page.cursor = aligned_offset + size;
        let ptr = unsafe { page.data.as_mut_ptr().add(aligned_offset) };
        pages.push(page);
        (ptr, size)
    }
}

fn align_up(offset: usize, align: usize) -> usize {
    (offset + align - 1) & !(align - 1)
}

impl Page {
    fn new(size: usize) -> Self {
        Page {
            data: vec![0u8; size],
            cursor: 0,
        }
    }
}

/// Reset token for scoped region clearing.
pub struct RegionReset {
    saved: Vec<usize>,
}

impl Region {
    /// Saves the current allocation state for later reset.
    pub fn checkpoint(&self) -> RegionReset {
        let saved = self.pages.borrow().iter().map(|p| p.cursor).collect();
        RegionReset { saved }
    }

    /// Resets the region to a previous checkpoint.
    pub fn reset_to(&self, checkpoint: &RegionReset) {
        let mut pages = self.pages.borrow_mut();
        for (i, &cursor) in checkpoint.saved.iter().enumerate() {
            if i < pages.len() {
                pages[i].cursor = cursor;
            }
        }
        pages.truncate(checkpoint.saved.len());
    }

    /// Resets the entire region to empty.
    pub fn reset(&self) {
        self.pages.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_alloc_int() {
        let region = Region::new();
        let x = region.alloc(42i32);
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_region_alloc_string() {
        let region = Region::new();
        let s = region.alloc(String::from("hello"));
        assert_eq!(*s, "hello");
    }

    #[test]
    fn test_region_alloc_slice() {
        let region = Region::new();
        let slice = region.alloc_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(slice.len(), 5);
        assert_eq!(slice[0], 1);
        assert_eq!(slice[4], 5);
    }

    #[test]
    fn test_region_reset() {
        let region = Region::new();
        let x = region.alloc(42i32);
        assert_eq!(*x, 42);
        region.reset();
        let y = region.alloc(100i32);
        assert_eq!(*y, 100);
    }

    #[test]
    fn test_region_checkpoint_reset() {
        let region = Region::new();
        let _a = region.alloc(1i32);
        let cp = region.checkpoint();
        let _b = region.alloc(2i32);
        region.reset_to(&cp);
        let c = region.alloc(3i32);
        assert_eq!(*c, 3);
    }
}
