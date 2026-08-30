//! NVMe endpoint BAR and command enable.
//!
//! Programs BAR0/1 as a 64-bit address pointing at CTRL_MEM_BASE
//! (0xA2000000) and enables memory space plus bus-master.

use tock_registers::interfaces::Writeable;

use super::{Bar0, Bar1, Command, PCIE_C_CTRL_NVME_CFG};

/// Point NVMe BAR0/1 at the MEM window and enable MEM+BME.
pub fn init() {
    PCIE_C_CTRL_NVME_CFG
        .bar_0
        .write(Bar0::BASE_ADDR::NVME_LOW + Bar0::MEMORY_TYPE::RANGE_64_BIT);
    PCIE_C_CTRL_NVME_CFG.bar_1.write(Bar1::BASE_ADDR::NVME_HIGH);
    PCIE_C_CTRL_NVME_CFG
        .command
        .write(Command::MEMORY_ACCESS_ENABLE::SET + Command::BUS_MASTER_ENABLE::SET);
}
