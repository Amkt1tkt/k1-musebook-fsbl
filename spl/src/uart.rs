//! 16550 UART at 0xD4017000.
//!
//! Transmit waits for `TDRQ` then writes THR; `flush` waits for `TEMT`.

use super::MMIO;

mod operate;
mod register;

pub use self::operate::{flush, write_byte};
use self::register::{LineStatus, UART};
