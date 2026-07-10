//! I/O operations for Gala programs.
//!
//! Provides console output, file reading, and file writing primitives.

use gala_string::GalaString;

/// Result type for I/O operations.
pub type IoResult<T> = Result<T, IoError>;

/// Errors that can occur during I/O operations.
#[derive(Debug)]
pub enum IoError {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    InvalidInput,
    Other(String),
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::NotFound => write!(f, "file not found"),
            IoError::PermissionDenied => write!(f, "permission denied"),
            IoError::AlreadyExists => write!(f, "file already exists"),
            IoError::InvalidInput => write!(f, "invalid input"),
            IoError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

/// Prints a string to stdout.
pub fn print(s: &GalaString) {
    print!("{}", s);
}

/// Prints a string to stdout followed by a newline.
pub fn println(s: &GalaString) {
    println!("{}", s);
}

/// Prints a formatted string to stdout.
pub fn print_fmt(args: std::fmt::Arguments<'_>) {
    print!("{args}");
}

/// Reads the entire contents of a file into a `GalaString`.
pub fn read_to_string(path: &GalaString) -> IoResult<GalaString> {
    match std::fs::read_to_string(path.as_str()) {
        Ok(s) => Ok(GalaString::from(s.as_str())),
        Err(e) => Err(match e.kind() {
            std::io::ErrorKind::NotFound => IoError::NotFound,
            std::io::ErrorKind::PermissionDenied => IoError::PermissionDenied,
            _ => IoError::Other(e.to_string()),
        }),
    }
}

/// Writes a string to a file, creating or truncating it.
pub fn write(path: &GalaString, content: &GalaString) -> IoResult<()> {
    std::fs::write(path.as_str(), content.as_str())
        .map_err(|e| IoError::Other(e.to_string()))
}

/// Appends a string to a file.
pub fn append(path: &GalaString, content: &GalaString) -> IoResult<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path.as_str())
        .map_err(|e| IoError::Other(e.to_string()))?;
    file.write_all(content.as_str().as_bytes())
        .map_err(|e| IoError::Other(e.to_string()))
}

/// Reads a line from stdin.
pub fn read_line() -> IoResult<GalaString> {
    let mut input = std::string::String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => Ok(GalaString::from(input.trim())),
        Err(e) => Err(IoError::Other(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        assert_eq!(format!("{}", IoError::NotFound), "file not found");
        assert_eq!(format!("{}", IoError::PermissionDenied), "permission denied");
    }

    #[test]
    fn test_write_and_read() {
        let path = GalaString::from("/tmp/_gala_io_test.txt");
        let content = GalaString::from("hello gala");
        assert!(write(&path, &content).is_ok());
        let read_back = read_to_string(&path);
        assert!(read_back.is_ok());
        assert_eq!(read_back.unwrap().as_str(), "hello gala");
        let _ = std::fs::remove_file("/tmp/_gala_io_test.txt");
    }

    #[test]
    fn test_read_nonexistent() {
        let path = GalaString::from("/tmp/_nonexistent_file_xyz.gala");
        match read_to_string(&path) {
            Err(IoError::NotFound) => {}
            _ => panic!("expected NotFound error"),
        }
    }
}
