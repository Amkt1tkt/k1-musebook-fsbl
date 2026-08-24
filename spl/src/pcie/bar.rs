use tock_registers::interfaces::Writeable;

use super::{Bar0, Bar1, Command, PCIE_C_CTRL_NVME_CFG};

pub fn init() {
    PCIE_C_CTRL_NVME_CFG
        .bar_0
        .write(Bar0::BASE_ADDR::NVME_LOW + Bar0::MEMORY_TYPE::RANGE_64_BIT);
    PCIE_C_CTRL_NVME_CFG.bar_1.write(Bar1::BASE_ADDR::NVME_HIGH);
    PCIE_C_CTRL_NVME_CFG
        .command
        .write(Command::MEMORY_ACCESS_ENABLE::SET + Command::BUS_MASTER_ENABLE::SET);
}
