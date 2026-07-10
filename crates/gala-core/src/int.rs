pub type Int = i64;

pub const ZERO: Int = 0;
pub const ONE: Int = 1;

pub fn add(a: Int, b: Int) -> Int {
    a + b
}

pub fn sub(a: Int, b: Int) -> Int {
    a - b
}

pub fn mul(a: Int, b: Int) -> Int {
    a * b
}

pub fn div(a: Int, b: Int) -> Int {
    a / b
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_sub() {
        assert_eq!(sub(5, 3), 2);
        assert_eq!(sub(1, 1), 0);
        assert_eq!(sub(0, 5), -5);
    }

    #[test]
    fn test_mul() {
        assert_eq!(mul(3, 4), 12);
        assert_eq!(mul(-2, 3), -6);
        assert_eq!(mul(0, 5), 0);
    }

    #[test]
    fn test_div() {
        assert_eq!(div(10, 3), 3);
        assert_eq!(div(7, 1), 7);
        assert_eq!(div(0, 5), 0);
    }

    #[test]
    fn test_constants() {
        assert_eq!(ZERO, 0);
        assert_eq!(ONE, 1);
    }
}