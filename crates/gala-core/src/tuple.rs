#[repr(C)]
pub struct Tuple2<T, U>(pub T, pub U);

#[repr(C)]
pub struct Tuple3<T, U, V>(pub T, pub U, pub V);

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuple2() {
        let t = Tuple2(42, "hello");
        assert_eq!(t.0, 42);
        assert_eq!(t.1, "hello");
    }

    #[test]
    fn test_tuple3() {
        let t = Tuple3(1, 2.5, true);
        assert_eq!(t.0, 1);
        assert!((t.1 - 2.5f64).abs() < 1e-10);
        assert!(t.2);
    }

    #[test]
    fn test_tuple2_repr() {
        // Verify repr(C) layout: fields are in order with no padding
        let t = Tuple2(0xABCDu16, 0x1234u16);
        let ptr = &t as *const _ as *const u16;
        unsafe {
            assert_eq!(*ptr, 0xABCD);
            assert_eq!(*ptr.add(1), 0x1234);
        }
    }
}
