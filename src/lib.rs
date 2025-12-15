#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

pub mod scd4x;
mod sensirion;
pub mod sgp40;

pub use sensirion::Error;
