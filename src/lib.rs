#![no_std]
#![allow(async_fn_in_trait)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(unsafe_code)]

pub(crate) mod commands;
pub mod driver;
pub mod errors;
pub mod graphics;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
