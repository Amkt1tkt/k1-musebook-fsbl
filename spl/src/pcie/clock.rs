//! Port C clock, reset, and PERST in root-complex mode.
//!
//! Selects RC, enables AXI DBI/slave/master clocks and deasserts their
//! resets, holds the PHY, sets IGNORE_PERSTN, then pulses RC_PERST for
//! 100 ms to hot-reset the endpoint.

use core::time::Duration;

use tock_registers::interfaces::ReadWriteable;

use super::{APMU, PciePortXClockResetControl, PciePortXControlLogic, time};

/// Configure Port C clocks/resets and pulse RC_PERST.
pub fn init() {
    APMU.pcie_port_c_clock_reset_control.modify({
        use PciePortXClockResetControl::*;
        PCIE_LTSSM_EN::CLEAR
            + PCIE_GLB_RST::CLEAR
            + PCIE_SYS_AUX_PWR_DET::CLEAR
            + PCIE_RC_PERST::CLEAR
            + PCIE_AXI_DBI_CLK_EN::SET
            + PCIE_AXI_SLV_CLK_EN::SET
            + PCIE_AXI_MSTR_CLK_EN::SET
            + PCIE_AXI_DBI_RESETN::SET
            + PCIE_AXI_SLV_RESETN::SET
            + PCIE_AXI_MSTR_RESETN::SET
            + PCIE_DEVICE_TYPE_SEL::SET
            + PCIE_APP_HOLD_PHY_RST::SET
    });
    APMU.pcie_port_c_control_logic
        .modify(PciePortXControlLogic::PCIE_IGNORE_PERSTN::SET);
    APMU.pcie_port_c_clock_reset_control
        .modify(PciePortXClockResetControl::PCIE_RC_PERST::SET);
    time::sleep(Duration::from_millis(100));
    APMU.pcie_port_c_clock_reset_control
        .modify(PciePortXClockResetControl::PCIE_RC_PERST::CLEAR);
}
