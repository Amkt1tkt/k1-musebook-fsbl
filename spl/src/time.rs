use super::MMIO;

mod operate;
mod register;

pub use self::operate::{init, sleep};
use self::register::{Control, GENERIC_COUNTER};
