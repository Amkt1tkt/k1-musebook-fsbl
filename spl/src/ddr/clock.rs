use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{
    APMU, ApClockControl, DdrCtrlAhb, DdrCtrlHardwareSleepType, DdrPhyPll1ControlLow,
    DdrPhyPll1Enable, freq::DdrFreq,
};

pub fn init() {
    config_pll_control();
    enable_pll();
    switch_freq_to_initial_1200_mt();
    enable_ahb_clock();
}

fn config_pll_control() {
    APMU.ddr_phy_pll_1_control_low
        .modify(DdrPhyPll1ControlLow::BYTE_1::MT_1200);
}

fn enable_pll() {
    APMU.ddr_phy_pll_1_enable.modify(
        DdrPhyPll1Enable::BIT_8::SET + DdrPhyPll1Enable::BIT_9::SET + DdrPhyPll1Enable::BIT_11::SET,
    );

    while !APMU
        .ddr_phy_pll_1_enable
        .matches_all(DdrPhyPll1Enable::BIT_16_17::SET)
    {
        core::hint::spin_loop();
    }
}

fn switch_freq_to_initial_1200_mt() {
    APMU.ddr_phy_pll_1_enable.write(DdrFreq::Mt1200.into());

    APMU.ddr_ctrl_hardware_sleep_type.write(
        DdrCtrlHardwareSleepType::DDRP_0_EN::SET
            + DdrCtrlHardwareSleepType::DCLK_BYPASS_CLK_EN::SET
            + DdrCtrlHardwareSleepType::DCLK_BYPASS_RST::SET
            + DdrCtrlHardwareSleepType::REG_TABLE_EN::SET,
    );

    APMU.ap_clock_control.write(ApClockControl::BIT_26::SET);
    while !APMU
        .ap_clock_control
        .matches_all(ApClockControl::BIT_26::CLEAR)
    {
        core::hint::spin_loop();
    }
}

fn enable_ahb_clock() {
    APMU.ddr_ctrl_ahb
        .modify(DdrCtrlAhb::AHBCLK_EN::SET + DdrCtrlAhb::HCLK_RST::SET);
}
