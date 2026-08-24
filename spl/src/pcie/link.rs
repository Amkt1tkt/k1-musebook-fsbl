use core::time::Duration;

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{
    APMU, Bar0, ClassCode, Command, InterruptPin, LinkCapabilities, LinkControl2, MiscControl1Off,
    PCIE_C_CTRL_DBI_CFG, PCIE_C_CTRL_DBI_PORT_LOGIC, PcieLinkWidthSpeedControl, PciePortDebug0,
    PciePortXClockResetControl, time,
};

pub fn init() {
    config_link_speed();
    setup_rc_bridge();
    ltssm();
}

fn config_link_speed() {
    let _guard = DbiRoWriteProtect::get_write_permission();
    PCIE_C_CTRL_DBI_CFG
        .link_capabilities
        .modify(LinkCapabilities::MAX_LINK_SPEED::GEN2);
    PCIE_C_CTRL_DBI_CFG
        .link_control_2
        .modify(LinkControl2::MAX_LINK_SPEED::GEN2);
}

fn setup_rc_bridge() {
    PCIE_C_CTRL_DBI_CFG
        .bar_0
        .write(Bar0::MEMORY_TYPE::RANGE_64_BIT);
    PCIE_C_CTRL_DBI_CFG.bar_1.set(0x0);

    PCIE_C_CTRL_DBI_CFG
        .interrupt_pin
        .write(InterruptPin::FULL::VALUE_01);

    PCIE_C_CTRL_DBI_CFG.primary_bus_number.set(0x0);
    PCIE_C_CTRL_DBI_CFG.secondary_bus_number.set(0x1);
    PCIE_C_CTRL_DBI_CFG.subordinate_bus_number.set(0xFF);

    PCIE_C_CTRL_DBI_CFG.command.write({
        use Command::*;
        IO_ACCESS_ENABLE::SET
            + MEMORY_ACCESS_ENABLE::SET
            + BUS_MASTER_ENABLE::SET
            + SERR_REPORTING_ENABLE::SET
    });

    let _guard = DbiRoWriteProtect::get_write_permission();
    PCIE_C_CTRL_DBI_CFG.class_code.write({
        use ClassCode::*;
        BASE_CLASS_CODE::VALUE_06 + SUB_CLASS_CODE::VALUE_04
    });
}

fn ltssm() {
    PCIE_C_CTRL_DBI_PORT_LOGIC
        .pcie_link_width_speed_control
        .modify(PcieLinkWidthSpeedControl::DIRECT_SPEED_CHANGE::SET);

    APMU.pcie_port_c_clock_reset_control.modify({
        use PciePortXClockResetControl::*;
        PCIE_LTSSM_EN::SET + PCIE_APP_HOLD_PHY_RST::CLEAR
    });

    while !PCIE_C_CTRL_DBI_PORT_LOGIC
        .pcie_port_debug_0
        .matches_all(PciePortDebug0::LTSSM_STATE::L0)
    {
        core::hint::spin_loop();
    }
    time::sleep(Duration::from_micros(100));
}

struct DbiRoWriteProtect;
impl DbiRoWriteProtect {
    fn get_write_permission() -> Self {
        PCIE_C_CTRL_DBI_PORT_LOGIC
            .misc_control_1_off
            .modify(MiscControl1Off::DBI_RO_WR_EN::SET);
        Self
    }
}
impl Drop for DbiRoWriteProtect {
    fn drop(&mut self) {
        PCIE_C_CTRL_DBI_PORT_LOGIC
            .misc_control_1_off
            .modify(MiscControl1Off::DBI_RO_WR_EN::CLEAR);
    }
}
