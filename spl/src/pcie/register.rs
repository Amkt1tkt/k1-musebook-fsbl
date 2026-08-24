use super::MMIO;

mod atu;
mod cfg;
mod dbi;
mod nvme;
mod phy;

pub use self::{
    atu::{
        LimitAddr, LowerBaseAddr, LowerTargetAddr, PCIE_C_CTRL_ATU_REGION_CFG,
        PCIE_C_CTRL_ATU_REGION_MEM, PcieCtrlAtu, RegionControl1, RegionControl2,
    },
    cfg::{
        Bar0, Bar1, ClassCode, Command, InterruptPin, LinkCapabilities, LinkControl2, PcieCtrlCfg,
    },
    dbi::{
        MiscControl1Off, PCIE_C_CTRL_DBI_CFG, PCIE_C_CTRL_DBI_PORT_LOGIC,
        PcieLinkWidthSpeedControl, PciePortDebug0,
    },
    nvme::{NVME_CTRL_BASE, PCIE_C_CTRL_NVME_CFG},
    phy::{
        ClockConfig, PCIE_A_PHY_LANE_0, PCIE_A_PHY_LANE_1, PCIE_C_PHY_LANE_0, PCIE_C_PHY_LANE_1,
        PciePhy, PllReg2, PllReg3, PllReg5, PuRxConfig, RcCalReg1, RcCalReg2, RefclkMode,
        RtermCalibrationResult, RtermCalibrationStatus, RxReg1, RxReg4, TxReg1, TxReg3,
    },
};

const CTRL_CFG_BASE: u32 = 0xA000_0000;
const CTRL_CFG_SIZE: u32 = 0x10_0000;
const CTRL_CFG_END: u32 = CTRL_CFG_BASE + CTRL_CFG_SIZE - 1;
const CTRL_IO_BASE: u32 = CTRL_CFG_BASE + CTRL_CFG_SIZE;
const CTRL_IO_SIZE: u32 = 0x10_0000;
const CTRL_MEM_BASE: u32 = CTRL_IO_BASE + CTRL_IO_SIZE;
const CTRL_MEM_SIZE: u32 = 0x1600_0000;
const CTRL_MEM_END: u32 = CTRL_MEM_BASE + CTRL_MEM_SIZE - 1;
