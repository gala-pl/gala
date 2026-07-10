//! UTF-8 string type and operations for Gala.
//!
//! Provides `String`, the heap-allocated growable UTF-8 string type,
//! along with construction, concatenation, slicing, and formatting operations.

use gala_core::int::Int;
use gala_core::bool::Bool;
use std::str::FromStr;

/// A heap-allocated, growable UTF-8 string.
pub struct GalaString {
    inner: std::string::String,
}

impl Default for GalaString {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for GalaString {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GalaString {
            inner: s.to_string(),
        })
    }
}

impl GalaString {
    /// Creates a new empty `GalaString`.
    pub fn new() -> Self {
        GalaString {
            inner: std::string::String::new(),
        }
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> Int {
        self.inner.len() as Int
    }

    /// Returns `true` if the string has length zero.
    pub fn is_empty(&self) -> Bool {
        self.inner.is_empty()
    }

    /// Appends a string slice to the end.
    pub fn push_str(&mut self, s: &GalaString) {
        self.inner.push_str(&s.inner);
    }

    /// Concatenates two strings, returning a new one.
    pub fn concat(&self, other: &GalaString) -> Self {
        let mut result = self.inner.clone();
        result.push_str(&other.inner);
        GalaString { inner: result }
    }

    /// Returns a slice of the string as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Checks if the string contains the given pattern.
    pub fn contains(&self, pattern: &GalaString) -> Bool {
        self.inner.contains(&pattern.inner)
    }

    /// Splits the string by whitespace, collecting into a vector.
    pub fn split_whitespace(&self) -> Vec<GalaString> {
        self.inner
            .split_whitespace()
            .map(GalaString::from_str)
            .collect::<Result<_, _>>()
            .unwrap()
    }
}

impl From<&str> for GalaString {
    fn from(s: &str) -> Self {
        GalaString {
            inner: s.to_string(),
        }
    }
}

impl std::fmt::Display for GalaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_string_is_empty() {
        let s = GalaString::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_from_str() {
        let s = GalaString::from("hello");
        assert!(!s.is_empty());
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_concat() {
        let a = GalaString::from("hello ");
        let b = GalaString::from("world");
        let c = a.concat(&b);
        assert_eq!(c.as_str(), "hello world");
    }

    #[test]
    fn test_contains() {
        let s = GalaString::from("hello world");
        assert!(s.contains(&GalaString::from("world")));
        assert!(!s.contains(&GalaString::from("xyz")));
    }
}
