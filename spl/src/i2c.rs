//! TWSI8 I2C master at 0xD401D800.
//!
//! `init` enables 204.8 MHz, PLL1 /5, GPIO118/119 AF2, then APBC-resets
//! and ungates the unit. `write` does a 7-bit master write (address << 1 |
//! write, START / data / STOP per byte, wait TX empty and ACK). `reset`
//! soft-resets, programs FAST master, and clears status.

use super::{
    APBC, APBS, ClockGating, MMIO, MPMU, PINMUX, Pinmux, PllXSw2Control, Twsi8ClockResetControl,
    time,
};

mod operate;
mod register;

pub use self::operate::{init, reset, write};
use self::register::{Control, I2C, Status};
