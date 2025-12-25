#![no_std]
#![allow(async_fn_in_trait)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(unsafe_code)]

mod commands;
pub mod driver;
pub mod errors;
pub mod graphics;
mod utils;
