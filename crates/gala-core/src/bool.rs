pub type Bool = bool;

pub const TRUE: Bool = true;
pub const FALSE: Bool = false;

pub fn and(a: Bool, b: Bool) -> Bool {
    a && b
}

pub fn or(a: Bool, b: Bool) -> Bool {
    a || b
}

pub fn not(a: Bool) -> Bool {
    !a
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and() {
        assert!(and(true, true));
        assert!(!and(true, false));
        assert!(!and(false, true));
        assert!(!and(false, false));
    }

    #[test]
    fn test_or() {
        assert!(or(true, true));
        assert!(or(true, false));
        assert!(or(false, true));
        assert!(!or(false, false));
    }

    #[test]
    fn test_not() {
        assert!(!not(true));
        assert!(not(false));
    }

    #[test]
    fn test_constants() {
        assert_eq!(TRUE, not(FALSE));
        assert_eq!(FALSE, not(TRUE));
    }
}
