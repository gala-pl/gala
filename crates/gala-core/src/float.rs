pub type Float = f64;

pub fn add(a: Float, b: Float) -> Float {
    a + b
}

pub fn sub(a: Float, b: Float) -> Float {
    a - b
}

pub fn mul(a: Float, b: Float) -> Float {
    a * b
}

pub fn div(a: Float, b: Float) -> Float {
    a / b
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert!((add(2.5, 3.1) - 5.6).abs() < 1e-10);
        assert!((add(-1.0, 1.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sub() {
        assert!((sub(5.5, 3.2) - 2.3).abs() < 1e-10);
    }

    #[test]
    fn test_mul() {
        assert!((mul(3.0, 1.5) - 4.5).abs() < 1e-10);
        assert_eq!(mul(0.0, 5.0), 0.0);
    }

    #[test]
    fn test_div() {
        assert!((div(10.0, 3.0) - 3.3333333333).abs() < 1e-9);
        assert!((div(0.0, 5.0) - 0.0).abs() < 1e-10);
    }
}
