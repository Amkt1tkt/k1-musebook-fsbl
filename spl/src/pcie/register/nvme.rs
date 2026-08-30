//! NVMe config header via iATU at the CFG base; controller MMIO at the MEM base.

use super::{CTRL_CFG_BASE, CTRL_MEM_BASE, MMIO, PcieCtrlCfg};

/// NVMe Type 0 config header (CFG window).
pub const PCIE_C_CTRL_NVME_CFG: MMIO<PcieCtrlCfg> = unsafe { MMIO::base(CTRL_CFG_BASE) };
/// NVMe controller MMIO base after BAR mapping (MEM window).
pub const NVME_CTRL_BASE: u32 = CTRL_MEM_BASE;
