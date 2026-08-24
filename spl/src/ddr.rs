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
mod mr;
mod phy;
mod register;
mod train;
mod verify;

use self::{
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
    ctrl::init(false);
    phy::init();
    dfi::handshake();
    dram::init();
    mr::init();
    train::train(DdrFreq::Mt1200);

    byte::prepare_reinit();

    // full init with byte mode
    ctrl::init(true);
    phy::init();
    dfi::handshake();
    dram::init();
    mr::init();

    ctrl::config_addr_mapping();
    freq::init_dynamic_freq_change();

    train::train(DdrFreq::Mt1200);
    freq::change_freq(DdrFreq::Mt1600);
    train::train(DdrFreq::Mt1600);
    freq::change_freq(DdrFreq::Mt2400);
    train::train(DdrFreq::Mt2400);

    mr::config_for_16gb();
    freq::change_freq(DdrFreq::Mt2400);

    verify::test_pattern();
}
