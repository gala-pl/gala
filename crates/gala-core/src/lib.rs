//! Core primitive types for Gala
//!
//! This crate provides the fundamental types used throughout the Gala compiler
//! and runtime.

#![no_std]

pub mod bool;
pub mod float;
pub mod int;
pub mod tuple;

pub type Int = i64;
pub type Float = f64;
pub type Bool = bool;
pub type Unit = ();
