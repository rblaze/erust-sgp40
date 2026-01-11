#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

pub mod scd4x;
mod sensirion;
pub mod sgp40;

#[cfg(test)]
mod debug_utils;

pub use sensirion::Error;
