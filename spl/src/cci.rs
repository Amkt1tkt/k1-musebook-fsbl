use super::MMIO;

mod operate;
mod register;

pub use operate::enable_snoop;
pub use register::CCI;
use register::{SnoopControl, Status};
