//! Outbound iATU windows for CFG and MEM.
//!
//! Two outbound regions: Cfg maps 0xA0000000 as CFG0 (target 0x01000000);
//! Mem maps 0xA2000000 as MEM. After programming, wait until REGION_EN sticks.

use core::time::Duration;

use tock_registers::{
    fields::FieldValue,
    interfaces::{Readable, Writeable},
};

use super::{
    LimitAddr, LowerBaseAddr, LowerTargetAddr, MMIO, PCIE_C_CTRL_ATU_REGION_CFG,
    PCIE_C_CTRL_ATU_REGION_MEM, PcieCtrlAtu, RegionControl1, RegionControl2, time,
};

/// Program the Cfg and Mem outbound iATU regions.
pub fn init() {
    program_atu(Region::Cfg);
    program_atu(Region::Mem);
}

/// Write one outbound region and wait for REGION_EN.
fn program_atu(region: Region) {
    let atu: MMIO<PcieCtrlAtu> = region.into();
    atu.lower_base_addr.write(region.into());
    atu.upper_base_addr.set(0x0);
    atu.limit_addr.write(region.into());
    atu.lower_target_addr.write(region.into());
    atu.upper_target_addr.set(0x0);
    atu.region_control_1.write(region.into());
    atu.region_control_2.write(RegionControl2::REGION_EN::SET);

    while !atu
        .region_control_2
        .matches_all(RegionControl2::REGION_EN::SET)
    {
        time::sleep(Duration::from_millis(10));
    }
}

/// Outbound iATU region (Cfg or Mem).
#[derive(Clone, Copy)]
enum Region {
    /// CFG0 window at 0xA0000000, target 0x01000000.
    Cfg,
    /// MEM window at 0xA2000000.
    Mem,
}

impl From<Region> for MMIO<PcieCtrlAtu> {
    fn from(value: Region) -> Self {
        match value {
            Region::Cfg => PCIE_C_CTRL_ATU_REGION_CFG,
            Region::Mem => PCIE_C_CTRL_ATU_REGION_MEM,
        }
    }
}

impl From<Region> for FieldValue<u32, LowerBaseAddr::Register> {
    fn from(value: Region) -> Self {
        match value {
            Region::Cfg => LowerBaseAddr::ADDR::CFG,
            Region::Mem => LowerBaseAddr::ADDR::MEM,
        }
    }
}

impl From<Region> for FieldValue<u32, LimitAddr::Register> {
    fn from(value: Region) -> Self {
        match value {
            Region::Cfg => LimitAddr::ADDR::CFG,
            Region::Mem => LimitAddr::ADDR::MEM,
        }
    }
}

impl From<Region> for FieldValue<u32, LowerTargetAddr::Register> {
    fn from(value: Region) -> Self {
        match value {
            Region::Cfg => LowerTargetAddr::ADDR::CFG,
            Region::Mem => LowerTargetAddr::ADDR::MEM,
        }
    }
}

impl From<Region> for FieldValue<u32, RegionControl1::Register> {
    fn from(value: Region) -> Self {
        match value {
            Region::Cfg => RegionControl1::TYPE::CFG0,
            Region::Mem => RegionControl1::TYPE::MEM,
        }
    }
}
