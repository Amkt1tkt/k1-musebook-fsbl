use core::time::Duration;

use tock_registers::interfaces::ReadWriteable;

use super::{APMU, DDR_CTRL_CHANNEL, DDR_CTRL_PHY_CONTROL, DdrCtrlHardwareSleepType, time};

pub fn prepare_reinit() {
    reset_dclk_bypass_clock();
}

fn reset_dclk_bypass_clock() {
    APMU.ddr_ctrl_hardware_sleep_type
        .modify(DdrCtrlHardwareSleepType::DCLK_BYPASS_RST::CLEAR);
    time::sleep(Duration::from_micros(100));
    APMU.ddr_ctrl_hardware_sleep_type
        .modify(DdrCtrlHardwareSleepType::DCLK_BYPASS_RST::SET);
    time::sleep(Duration::from_micros(100));
}

pub fn set_byte_mode_parameter() {
    unsafe {
        // FP 3
        DDR_CTRL_CHANNEL.write([(0x0104, 0xF080_0400)]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x1900_0A02)]);
        DDR_CTRL_CHANNEL.write([(0x01B4, 0x0800_1400)]);
        // FP 2
        DDR_CTRL_CHANNEL.write([(0x0104, 0xA080_0400)]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x1500_0802)]);
        DDR_CTRL_CHANNEL.write([(0x01B4, 0x0800_1000)]);
        // FP 1
        DDR_CTRL_CHANNEL.write([(0x0104, 0x5080_0400)]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x0C00_0402)]);
        DDR_CTRL_CHANNEL.write([(0x01B4, 0x0800_0A00)]);
        // FP 0
        DDR_CTRL_CHANNEL.write([(0x0104, 0x0080_0400)]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x0C00_0402)]);
        DDR_CTRL_CHANNEL.write([(0x01B4, 0x0800_0A00)]);
    }
}
