//! Core of AEP Downgrader: parse After Effects project files (RIFX), detect
//! their version and rewrite them for an older After Effects.
//!
//! Rules for this crate:
//! - no file or network I/O: every function works on byte slices, so the crate
//!   also builds for `wasm32-unknown-unknown`;
//! - no dependencies outside `std`;
//! - no panics on malformed input: return an error instead.

pub mod riff;
