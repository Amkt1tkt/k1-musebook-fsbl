//! NXP-style FlexSPI/QSPI NOR at `0xD420C000` with AHB window `0xB8000000`.
//!
//! `init` programs pinmux, clocks, a soft reset, LUT0 `0x0B` Fast Read for AHB,
//! and LUT1 for IP commands. `read` is an AHB memcpy; `write` is WREN+PP in
//! 256-byte pages; `erase` is 4K SE. Writes and erases invalidate the AHB
//! cache (`SWRSTHD`/`SWRSTSD`).

use super::FlashServerError;

mod operate;
mod register;

pub use self::operate::{erase, init, read, write};
use self::register::{Fr, Ipcr, Lckcr, Lutkey, Mcr, QSPI, Sptrclr, Sr};
