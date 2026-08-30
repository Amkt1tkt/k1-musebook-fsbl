//! K1 DDR bring-up: Marvell-lineage memory controller plus in-house PHY.
//!
//! This is not Synopsys uMCTL2. `init` runs two passes:
//!
//! 1. Clock → controller (byte-mode off) → PHY → DFI handshake → DRAM MR init → 1200 MT train.
//! 2. Read MR8/MR5 for capacity (8/16 GB) and manufacturer.
//! 3. `prepare_reinit` resets DCLK bypass, then a full reconfig with byte-mode and Hynix tweaks.
//! 4. Address map and dynamic frequency table by capacity.
//! 5. Train 1200 → switch and train 1600 → switch and train 2400 → extra MR on 16 GB → switch 2400 again → pattern check at 0x10000.

use super::{
    APMU, ApClockControl, ApInterruptMask, DDR_TRAIN_VERIFY_BASE, DdrCtrlAhb,
    DdrCtrlHardwareSleepType, DdrPhyLdoControl, DdrPhyPll1ControlLow, DdrPhyPll1Enable,
    DdrPhyPllDiv, MMIO, Raw, cpu, time,
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

/// Two-pass DDR bring-up, finishing at 2400 MT with a pattern check at 0x10000.
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
