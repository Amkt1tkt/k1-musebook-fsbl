use super::{APMU, MMIO, PciePortXClockResetControl, PciePortXControlLogic, time};

mod atu;
mod bar;
mod clock;
mod link;
mod phy;
mod register;

pub use self::register::NVME_CTRL_BASE;
use self::register::{
    Bar0, Bar1, ClassCode, ClockConfig, Command, InterruptPin, LimitAddr, LinkCapabilities,
    LinkControl2, LowerBaseAddr, LowerTargetAddr, MiscControl1Off, PCIE_A_PHY_LANE_0,
    PCIE_A_PHY_LANE_1, PCIE_C_CTRL_ATU_REGION_CFG, PCIE_C_CTRL_ATU_REGION_MEM, PCIE_C_CTRL_DBI_CFG,
    PCIE_C_CTRL_DBI_PORT_LOGIC, PCIE_C_CTRL_NVME_CFG, PCIE_C_PHY_LANE_0, PCIE_C_PHY_LANE_1,
    PcieCtrlAtu, PcieLinkWidthSpeedControl, PciePhy, PciePortDebug0, PllReg2, PllReg3, PllReg5,
    PuRxConfig, RcCalReg1, RcCalReg2, RefclkMode, RegionControl1, RegionControl2,
    RtermCalibrationResult, RtermCalibrationStatus, RxReg1, RxReg4, TxReg1, TxReg3,
};

pub fn init() {
    log::info!("pcie init");
    clock::init();
    phy::init();
    link::init();
    atu::init();
    bar::init();
}
