use super::MMIO;

mod operate;
mod register;

pub use self::operate::{flush, write_byte};
use self::register::{LineStatus, UART};
