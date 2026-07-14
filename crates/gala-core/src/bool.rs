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
        assert_eq!(and(true, true), true);
        assert_eq!(and(true, false), false);
        assert_eq!(and(false, true), false);
        assert_eq!(and(false, false), false);
    }

    #[test]
    fn test_or() {
        assert_eq!(or(true, true), true);
        assert_eq!(or(true, false), true);
        assert_eq!(or(false, true), true);
        assert_eq!(or(false, false), false);
    }

    #[test]
    fn test_not() {
        assert_eq!(not(true), false);
        assert_eq!(not(false), true);
    }

    #[test]
    fn test_constants() {
        assert_eq!(TRUE, true);
        assert_eq!(FALSE, false);
    }
}
