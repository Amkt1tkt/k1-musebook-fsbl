//! Generic Counter at 0xD5001000: enable, then busy-wait sleep.
//!
//! A 64-bit read retries until the high half is stable.

use super::MMIO;

mod operate;
mod register;

pub use self::operate::{init, sleep};
use self::register::{Control, GENERIC_COUNTER};
