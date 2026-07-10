//! Collection data structures for Gala programs.
//!
//! Provides `Vec<T>`, `HashMap<K, V>`, and `HashSet<T>` wrappers
//! around their Rust standard library counterparts.

use gala_core::int::Int;
use gala_core::bool::Bool;
use std::hash::Hash;
use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};

/// A contiguous growable array type.
pub struct GVec<T> {
    inner: Vec<T>,
}

impl<T> Default for GVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GVec<T> {
    /// Creates a new empty vector.
    pub fn new() -> Self {
        GVec { inner: Vec::new() }
    }

    /// Creates a new empty vector with the given capacity.
    pub fn with_capacity(capacity: Int) -> Self {
        GVec {
            inner: Vec::with_capacity(capacity as usize),
        }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> Int {
        self.inner.len() as Int
    }

    /// Returns `true` if the vector is empty.
    pub fn is_empty(&self) -> Bool {
        self.inner.is_empty()
    }

    /// Appends an element to the back.
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// Removes the last element and returns it.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Returns a reference to an element at the given index.
    pub fn get(&self, index: Int) -> Option<&T> {
        self.inner.get(index as usize)
    }

    /// Returns a mutable reference to an element at the given index.
    pub fn get_mut(&mut self, index: Int) -> Option<&mut T> {
        self.inner.get_mut(index as usize)
    }

    /// Returns the current capacity.
    pub fn capacity(&self) -> Int {
        self.inner.capacity() as Int
    }

    /// Reserves capacity for at least `additional` more elements.
    pub fn reserve(&mut self, additional: Int) {
        self.inner.reserve(additional as usize);
    }

    /// Iterates over elements.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.inner.iter()
    }

    /// Sorts the vector.
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.inner.sort();
    }

    /// Reverses the vector in place.
    pub fn reverse(&mut self) {
        self.inner.reverse();
    }
}

impl<T> From<Vec<T>> for GVec<T> {
    fn from(v: Vec<T>) -> Self {
        GVec { inner: v }
    }
}

impl<T> From<GVec<T>> for Vec<T> {
    fn from(val: GVec<T>) -> Self {
        val.inner
    }
}

/// A hash map from keys `K` to values `V`.
pub struct GHashMap<K, V> {
    inner: StdHashMap<K, V>,
}

impl<K, V> Default for GHashMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> GHashMap<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new empty hash map.
    pub fn new() -> Self {
        GHashMap {
            inner: StdHashMap::new(),
        }
    }

    /// Inserts a key-value pair, returning the previous value if any.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// Returns a reference to the value for the given key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    /// Returns `true` if the map contains the given key.
    pub fn contains_key(&self, key: &K) -> Bool {
        self.inner.contains_key(key)
    }

    /// Returns the number of entries.
    pub fn len(&self) -> Int {
        self.inner.len() as Int
    }

    /// Returns `true` if the map is empty.
    pub fn is_empty(&self) -> Bool {
        self.inner.is_empty()
    }

    /// Removes a key, returning the value if present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    /// Iterates over key-value pairs.
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.inner.iter()
    }
}

/// A hash set of values `T`.
pub struct GHashSet<T> {
    inner: StdHashSet<T>,
}

impl<T> Default for GHashSet<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GHashSet<T>
where
    T: Eq + Hash,
{
    /// Creates a new empty hash set.
    pub fn new() -> Self {
        GHashSet {
            inner: StdHashSet::new(),
        }
    }

    /// Inserts a value, returning `true` if it was not already present.
    pub fn insert(&mut self, value: T) -> Bool {
        self.inner.insert(value)
    }

    /// Returns `true` if the set contains the given value.
    pub fn contains(&self, value: &T) -> Bool {
        self.inner.contains(value)
    }

    /// Returns the number of elements.
    pub fn len(&self) -> Int {
        self.inner.len() as Int
    }

    /// Returns `true` if the set is empty.
    pub fn is_empty(&self) -> Bool {
        self.inner.is_empty()
    }

    /// Removes a value, returning `true` if it was present.
    pub fn remove(&mut self, value: &T) -> Bool {
        self.inner.remove(value)
    }

    /// Iterates over elements.
    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, T> {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_push_pop() {
        let mut v = GVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);
        assert_eq!(*v.get(0).unwrap(), 1);
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_vec_sort() {
        let mut v = GVec::from(vec![3, 1, 2]);
        v.sort();
        assert_eq!(*v.get(0).unwrap(), 1);
        assert_eq!(*v.get(1).unwrap(), 2);
        assert_eq!(*v.get(2).unwrap(), 3);
    }

    #[test]
    fn test_hashmap_insert_get() {
        let mut m = GHashMap::new();
        m.insert("a", 1);
        m.insert("b", 2);
        assert!(m.contains_key(&"a"));
        assert_eq!(*m.get(&"a").unwrap(), 1);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_hashset_insert_contains() {
        let mut s = GHashSet::new();
        assert!(s.insert(1));
        assert!(s.insert(2));
        assert!(!s.insert(1));
        assert!(s.contains(&1));
        assert!(!s.contains(&3));
        assert_eq!(s.len(), 2);
    }
}
