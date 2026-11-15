//! # Emberbound domain
//!
//! Pure game rules for Emberbound. This crate knows nothing about the engine,
//! rendering, input or the browser: every bounded context below is plain Rust
//! that can be unit-tested natively. Outer layers (the Bevy crates) depend on
//! this crate, never the other way round. Anything the domain needs from the
//! outside world is expressed as a port in [`ports`].

pub mod bestiary;
pub mod combat;
pub mod companion;
pub mod encounter;
pub mod ports;
pub mod progression;
pub mod world;
