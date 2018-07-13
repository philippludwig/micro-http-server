#![deny(missing_docs)]

//! Crate doc

#[macro_use] extern crate log;

/// Documentation?
mod microhttp;
mod request;

pub use microhttp::MicroHTTP;

#[cfg(target_os="linux")]
fn os_windows() -> bool { false }

#[cfg(target_os="windows")]
fn os_windows() -> bool { true }
