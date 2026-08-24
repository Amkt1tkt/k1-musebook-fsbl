use super::{CTRL_CFG_BASE, CTRL_MEM_BASE, MMIO, PcieCtrlCfg};

pub const PCIE_C_CTRL_NVME_CFG: MMIO<PcieCtrlCfg> = unsafe { MMIO::base(CTRL_CFG_BASE) };
pub const NVME_CTRL_BASE: u32 = CTRL_MEM_BASE;
