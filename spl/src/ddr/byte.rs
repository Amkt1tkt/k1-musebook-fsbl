//! x8 byte-mode timing and DCLK-bypass reset between the two bring-up passes.
//!
//! x8 devices need wider read timing at channel 0x1B4 and PHY 0x3E4.
//! `prepare_reinit` pulses DCLK-bypass reset so the second `ddr::init` starts
//! from a clean clock state.

use core::time::Duration;

use tock_registers::interfaces::ReadWriteable;

use super::{APMU, DDR_CTRL_CHANNEL, DDR_CTRL_PHY_CONTROL, DdrCtrlHardwareSleepType, time};

/// Whether the second bring-up pass programs x8 byte-mode timing.
pub enum ByteMode {
    Enable,
    Disable,
}

/// Pulse DCLK-bypass reset before the second, full controller/PHY init.
pub fn prepare_reinit() {
    reset_dclk_bypass_clock();
}

/// Clear then set DCLK-bypass reset, 100 µs each side.
fn reset_dclk_bypass_clock() {
    APMU.ddr_ctrl_hardware_sleep_type
        .modify(DdrCtrlHardwareSleepType::DCLK_BYPASS_RST::CLEAR);
    time::sleep(Duration::from_micros(100));
    APMU.ddr_ctrl_hardware_sleep_type
        .modify(DdrCtrlHardwareSleepType::DCLK_BYPASS_RST::SET);
    time::sleep(Duration::from_micros(100));
}

/// Widen read timing (0x1B4 / PHY 0x3E4) at each of the four frequency points.
pub fn set_byte_mode_parameter() {
    unsafe {
        // 3200 MT
        DDR_CTRL_CHANNEL.write([(0x0104, 0xF080_0400)]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x1900_0A02)]);
        DDR_CTRL_CHANNEL.write([(0x01B4, 0x0800_1400)]);
        // 2400 MT
        DDR_CTRL_CHANNEL.write([(0x0104, 0xA080_0400)]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x1500_0802)]);
        DDR_CTRL_CHANNEL.write([(0x01B4, 0x0800_1000)]);
        // 1600 MT
        DDR_CTRL_CHANNEL.write([(0x0104, 0x5080_0400)]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x0C00_0402)]);
        DDR_CTRL_CHANNEL.write([(0x01B4, 0x0800_0A00)]);
        // 1200 MT
        DDR_CTRL_CHANNEL.write([(0x0104, 0x0080_0400)]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x0C00_0402)]);
        DDR_CTRL_CHANNEL.write([(0x01B4, 0x0800_0A00)]);
    }
}
