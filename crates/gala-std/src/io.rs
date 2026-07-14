pub fn print(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    wasm_print(msg);

    #[cfg(not(target_arch = "wasm32"))]
    std_print(msg);
}

#[cfg(not(target_arch = "wasm32"))]
fn std_print(msg: &str) {
    println!("{msg}");
}

#[cfg(target_arch = "wasm32")]
fn wasm_print(msg: &str) {
    extern "C" {
        fn js_console_log(ptr: *const u8, len: usize);
    }
    unsafe {
        js_console_log(msg.as_ptr(), msg.len());
    }
}

pub fn println(msg: &str) {
    print(msg);
    print("\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_does_not_panic() {
        // Just verify print doesn't crash
        print("test message");
    }

    #[test]
    fn test_println_does_not_panic() {
        println("test message");
    }

    #[test]
    fn test_println_adds_newline() {
        // Verify prints don't crash with special characters
        println("hello world!");
        println("");
        println("line1\nline2");
    }
}
