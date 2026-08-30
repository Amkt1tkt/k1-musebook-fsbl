//! ARM CCI interconnect for the two CPU clusters.
//!
//! `enable_snoop` turns on snoop + DVM and waits until `CHANGE_PENDING`
//! clears. Cluster 0's interface is at +0x1000; cluster 1 at +0x2000.
//! Cross-cluster coherency requires both sides.

use super::MMIO;

mod operate;
mod register;

pub use operate::enable_snoop;
pub use register::CCI;
use register::{SnoopControl, Status};
