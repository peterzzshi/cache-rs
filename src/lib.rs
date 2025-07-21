//! A generic cache implementation with expiration support
//! 
//! This crate provides a `Cache` struct that can store key-value pairs
//! with expiration times, automatically loading values when they expire
//! or are not present.

pub mod cache;

pub use cache::{Cache, Expiring};