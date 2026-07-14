use core::fmt;

#[repr(C)]
pub struct String {
    ptr: *const u8,
    len: usize,
}

impl String {
    pub fn new(s: &str) -> Self {
        String { ptr: s.as_ptr(), len: s.len() }
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.ptr, self.len)) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_string() {
        let s = String::new("hello");
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_empty_string() {
        let s = String::new("");
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_as_str() {
        let s = String::new("hello world");
        assert_eq!(s.as_str(), "hello world");
    }

    #[test]
    fn test_unicode_string() {
        let s = String::new("θ φ ψ");
        assert_eq!(s.as_str(), "θ φ ψ");
    }

    #[test]
    fn test_display() {
        let s = String::new("test");
        assert_eq!(format!("{}", s), "test");
    }

    #[test]
    fn test_repr_layout() {
        let msg = "hello";
        let s = String::new(msg);
        assert_eq!(s.as_str(), msg);
        assert_eq!(s.len(), msg.len());
    }
}
