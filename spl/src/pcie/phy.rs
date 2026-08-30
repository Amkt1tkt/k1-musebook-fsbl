//! PUPHY bring-up and Rterm calibration for Port C.
//!
//! K1 Rterm calibration hardware lives on Port A. This module clocks Port A
//! temporarily, programs 24 MHz ref / 100 MHz pipe with SSC off, waits for
//! A's rterm DONE, writes the result into both Port C lanes' RX/TX Rterm
//! registers and kicks RC cal, then shuts Port A, releases Port C PHY hold,
//! and waits for C's PLL lock. `TempPortA` Drop turns off A's clocks and
//! resets.

use core::time::Duration;

use tock_registers::{
    LocalRegisterCopy,
    interfaces::{ReadWriteable, Readable, Writeable},
    registers::ReadWrite,
};

use super::{
    APMU, ClockConfig, MMIO, PCIE_A_PHY_LANE_0, PCIE_A_PHY_LANE_1, PCIE_C_PHY_LANE_0,
    PCIE_C_PHY_LANE_1, PciePhy, PciePortXClockResetControl, PllReg2, PllReg3, PllReg5, PuRxConfig,
    RcCalReg1, RcCalReg2, RefclkMode, RtermCalibrationResult, RtermCalibrationStatus, RxReg1,
    RxReg4, TxReg1, TxReg3, time,
};

/// Port A PHY lane register blocks.
const PORT_A: [MMIO<PciePhy>; 2] = [PCIE_A_PHY_LANE_0, PCIE_A_PHY_LANE_1];
/// Port C PHY lane register blocks.
const PORT_C: [MMIO<PciePhy>; 2] = [PCIE_C_PHY_LANE_0, PCIE_C_PHY_LANE_1];

/// Calibrate on Port A, copy onto Port C, release hold, wait PLL lock.
pub fn init() {
    let shared_calibration_result = rterm_calibration_on_port_a();
    copy_calibration_result_to_port_c(shared_calibration_result);
    release_phy_hold(&APMU.pcie_port_c_clock_reset_control);
    config_port(PORT_C);
    wait_port_c_pll_lock();
}

/// Run Rterm calibration on Port A and return the result.
fn rterm_calibration_on_port_a() -> LocalRegisterCopy<u8, RtermCalibrationResult::Register> {
    let _guard = TempPortA::temp_enable_for_rterm_calibration();

    config_port(PORT_A);

    PCIE_C_PHY_LANE_0
        .pu_rx_config
        .modify(PuRxConfig::MPU_U3::SET + PuRxConfig::PU_RX_LFPS::SET);

    while !PCIE_A_PHY_LANE_0
        .rterm_calibration_status
        .matches_all(RtermCalibrationStatus::DONE::SET)
    {
        core::hint::spin_loop();
    }

    PCIE_A_PHY_LANE_0.rterm_calibration_result.extract()
}

/// Temporary Port A clock/reset enable; Drop shuts A back down.
struct TempPortA;
impl TempPortA {
    /// Enable Port A clocks/resets for Rterm calibration.
    fn temp_enable_for_rterm_calibration() -> Self {
        APMU.pcie_port_a_clock_reset_control.write({
            use PciePortXClockResetControl::*;
            PCIE_APP_HOLD_PHY_RST::SET
                + PCIE_AXI_DBI_CLK_EN::SET
                + PCIE_AXI_SLV_CLK_EN::SET
                + PCIE_AXI_MSTR_CLK_EN::SET
                + PCIE_AXI_DBI_RESETN::SET
                + PCIE_AXI_SLV_RESETN::SET
                + PCIE_AXI_MSTR_RESETN::SET
        });
        release_phy_hold(&APMU.pcie_port_a_clock_reset_control);
        Self
    }
}
impl Drop for TempPortA {
    fn drop(&mut self) {
        APMU.pcie_port_a_clock_reset_control.modify({
            use PciePortXClockResetControl::*;
            PCIE_APP_HOLD_PHY_RST::CLEAR
                + PCIE_AXI_DBI_CLK_EN::CLEAR
                + PCIE_AXI_SLV_CLK_EN::CLEAR
                + PCIE_AXI_MSTR_CLK_EN::CLEAR
                + PCIE_AXI_DBI_RESETN::CLEAR
                + PCIE_AXI_SLV_RESETN::CLEAR
                + PCIE_AXI_MSTR_RESETN::CLEAR
        });
    }
}

/// Write A's Rterm result into both Port C lanes and trigger RC cal.
fn copy_calibration_result_to_port_c(
    result: LocalRegisterCopy<u8, RtermCalibrationResult::Register>,
) {
    PORT_C.iter().for_each(|lane| {
        lane.rx_reg1.modify(
            RxReg1::RTERM_CALIBRATION_LSB
                .val(result.read(RtermCalibrationResult::RTERM_CALIBRATION_LSB)),
        );
        lane.rx_reg4.modify(RxReg4::BIT_5::CLEAR);
        lane.tx_reg1.modify(
            TxReg1::RTERM_CALIBRATION_MSB
                .val(result.read(RtermCalibrationResult::RTERM_CALIBRATION_MSB)),
        );
        lane.tx_reg3.modify(TxReg3::BIT_1::SET);
        lane.rc_cal_reg1.modify(RcCalReg1::BIT_6::CLEAR);
        lane.rc_cal_reg1.modify(RcCalReg1::BIT_6::SET);
    });
    PCIE_C_PHY_LANE_0
        .rc_cal_reg2
        .modify(RcCalReg2::CAL_REFCLK_FREQ::MHZ_24);
    PCIE_C_PHY_LANE_0
        .pu_rx_config
        .modify(PuRxConfig::PU_RX_LFPS::CLEAR + PuRxConfig::MPU_U3::CLEAR);
}

/// Spin until Port C PHY PLL reports lock.
fn wait_port_c_pll_lock() {
    time::sleep(Duration::from_millis(1));

    while !PCIE_C_PHY_LANE_0
        .clock_config
        .matches_all(ClockConfig::BIT_0::SET)
    {
        core::hint::spin_loop();
    }
}

/// Program 24 MHz refclk, 100 MHz pipe, and disable SSC.
fn config_port(lanes: [MMIO<PciePhy>; 2]) {
    lanes.iter().for_each(|lane| {
        lane.refclk_mode.modify(
            RefclkMode::ENABLE::SET + RefclkMode::DRIVER::SET + RefclkMode::RECEIVER::CLEAR,
        );
    });

    lanes[0].pll_reg2.modify(PllReg2::INPUT_FREQ::REFCLK_MHZ_24);
    lanes[0].pll_reg5.modify(PllReg5::OUTPUT_FREQ_MHZ_100::SET);

    lanes[0].pll_reg3.modify(PllReg3::SSC_ENABLE::CLEAR);
    lanes.iter().for_each(|lane| {
        lane.clock_config.write(ClockConfig::FULL::VALUE_0B78);
    });
    lanes[0]
        .pu_rx_config
        .write(PuRxConfig::FORCE_RECIVE_DONE::SET);
}

/// Clear `PCIE_APP_HOLD_PHY_RST` on the given port.
fn release_phy_hold(port: &ReadWrite<u32, PciePortXClockResetControl::Register>) {
    port.modify(PciePortXClockResetControl::PCIE_APP_HOLD_PHY_RST::CLEAR);
}
