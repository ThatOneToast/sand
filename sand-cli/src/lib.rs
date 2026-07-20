#![forbid(unsafe_code)]

//! Internal support library for the `sand` CLI.
//!
//! The public surface here exists so integration benchmarks and the binary can
//! use the same build and sync helpers without path-importing CLI source files.

#[doc(hidden)]
pub mod add_cmd;
#[doc(hidden)]
pub mod build;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod console;
#[doc(hidden)]
pub mod join_cmd;
#[doc(hidden)]
pub mod pack_format;
#[doc(hidden)]
pub mod run_cmd;
#[doc(hidden)]
pub mod scaffold;
