use super::{
    APMU, ApClockControl, ApInterruptMask, DdrCtrlAhb, DdrCtrlHardwareSleepType, DdrPhyLdoControl,
    DdrPhyPll1ControlLow, DdrPhyPll1Enable, DdrPhyPllDiv, MMIO, Raw, cpu, time,
};

mod byte;
mod clock;
mod ctrl;
mod dfi;
mod dram;
mod freq;
mod image;
mod phy;
mod register;
mod train;
mod verify;

use self::{
    byte::ByteMode,
    dram::DdrCapacity,
    freq::DdrFreq,
    register::{
        DDR_CTRL, DDR_CTRL_BASE, DDR_CTRL_CHANNEL, DDR_CTRL_CHANNEL_OFFSET, DDR_CTRL_PHY_CONTROL,
        DDR_CTRL_SECURE_ALIAS, DDR_PHY, DDR_PHY_FREQ_POINT_STEP, DDR_PHY_OTHER_CONTROL,
        DDR_PHY_SUB_A, DDR_PHY_SUB_B,
    },
};

pub fn init() {
    log::info!("ddr init");
    clock::init();

    // early init without byte mode
    ctrl::init(ByteMode::Disable);
    phy::init();
    dfi::handshake();
    dram::init();
    train::train(DdrFreq::Mt1200);

    let capacity = dram::detect_capacity();
    let manufacturer = dram::detect_manufacturer();

    byte::prepare_reinit();

    // full init with byte mode
    ctrl::init(ByteMode::Enable);
    phy::init();
    phy::config_for_manufacturer(manufacturer);
    dfi::handshake();
    dram::init();

    ctrl::config_addr_mapping(capacity);
    freq::init_dynamic_freq_change(capacity);

    train::train(DdrFreq::Mt1200);
    freq::change_freq(DdrFreq::Mt1600);
    train::train(DdrFreq::Mt1600);
    freq::change_freq(DdrFreq::Mt2400);
    train::train(DdrFreq::Mt2400);

    dram::config_for_capacity(capacity);

    freq::change_freq(DdrFreq::Mt2400);

    verify::test_pattern();
}
