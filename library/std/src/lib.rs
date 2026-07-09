//! Standard library for the Gala programming language.
//!
//! This umbrella crate re-exports the standard library crates under
//! a single `gala_std` namespace for convenience.

pub use gala_core;
pub use gala_string as string;
pub use gala_io as io;
pub use gala_collections as collections;
pub use gala_chan as chan;
pub use gala_region as region;
