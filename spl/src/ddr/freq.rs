use tock_registers::{
    fields::FieldValue,
    interfaces::{ReadWriteable, Readable, Writeable},
};

use super::{
    APMU, ApClockControl, ApInterruptMask, DDR_CTRL, DDR_CTRL_CHANNEL, DDR_CTRL_CHANNEL_OFFSET,
    DDR_CTRL_SECURE_ALIAS, DDR_PHY, DDR_PHY_FREQ_POINT_STEP, DdrCtrlHardwareSleepType,
    DdrPhyPll1Enable,
};

pub fn init_dynamic_freq_change() {
    config_dynamic_freq_change_table();
    config_ddr_ctrl_timing_table();
}

pub fn change_freq(freq: DdrFreq) {
    APMU.ap_interrupt_mask
        .modify(ApInterruptMask::DCLK_FC_DONE_INT_MSK::SET);

    unsafe {
        DDR_CTRL.write([(0x148, 0x80AC_0000)]);
    }

    APMU.ddr_phy_pll_1_enable.write(freq.into());
    APMU.ddr_ctrl_hardware_sleep_type.write(freq.into());

    APMU.ap_clock_control
        .write(ApClockControl::DDR_FREQ_CHG_REQ::SET);
    while !APMU
        .ap_clock_control
        .matches_all(ApClockControl::DDR_FREQ_CHG_REQ::CLEAR)
    {
        core::hint::spin_loop();
    }
}

#[derive(Clone, Copy)]
pub enum DdrFreq {
    Mt1200 = 0,
    Mt1600 = 1,
    Mt2400 = 2,
    Mt3200 = 3,
    ExternalClock = 4,
}

impl From<DdrFreq> for FieldValue<u32, DdrPhyPll1Enable::Register> {
    fn from(value: DdrFreq) -> Self {
        use DdrPhyPll1Enable::FREQ::*;
        match value {
            DdrFreq::Mt1200 => MT_1200,
            DdrFreq::Mt1600 => MT_1600,
            DdrFreq::Mt2400 => MT_2400,
            DdrFreq::Mt3200 => MT_3200,
            DdrFreq::ExternalClock => EXTERNAL_CLOCK,
        }
    }
}

impl From<DdrFreq> for FieldValue<u32, DdrCtrlHardwareSleepType::Register> {
    fn from(value: DdrFreq) -> Self {
        use DdrCtrlHardwareSleepType::*;
        (DDRP_0_EN::SET + DCLK_BYPASS_RST::SET)
            + match value {
                DdrFreq::ExternalClock => DCLK_BYPASS_CLK_EN::SET,
                _ => FREQ_PLL_CHG_MODE::SET,
            }
            + match value {
                DdrFreq::Mt1200 => REG_TABLE_NUM::MT_1200,
                DdrFreq::Mt1600 => REG_TABLE_NUM::MT_1600,
                DdrFreq::Mt2400 => REG_TABLE_NUM::MT_2400,
                DdrFreq::Mt3200 => REG_TABLE_NUM::MT_3200,
                DdrFreq::ExternalClock => DCLK_BYPASS_DIV_BIT_16::SET,
            }
    }
}

impl DdrFreq {
    pub fn phy_offset(&self) -> u32 {
        *self as u32 * DDR_PHY_FREQ_POINT_STEP
    }
}

fn config_dynamic_freq_change_table() {
    ddr_ctrl_secure_alias_write_table(
        0x0,
        [
            (0x0004_0303, 0x0000_0044),
            (0x1300_0008, 0x0000_0020),
            (0x1301_0000, 0x0000_0028),
            (0x1302_000D, 0x0000_0024),
            (0x1302_0001, 0x0000_0024),
            (0x1302_0002, 0x0000_0024),
            (0x1302_0003, 0x0000_0024),
            (0x1302_000B, 0x0000_0024),
            (0x1302_000C, 0x0000_0024),
            (0x1302_000E, 0x0000_0024),
            (0x1302_0016, 0x0000_0024),
            (0x1300_8000, 0x0000_0028),
            (0x1302_000D, 0x0000_0024),
            (0x1300_0010, 0x0000_0020),
            (0x0000_0002, 0x0000_2008),
            (0x0000_0002, 0x0000_2008),
            (0x1300_0001, 0x0000_13D0),
            (0x0000_8000, 0x0000_33FC),
            (0x0000_0000, 0x0000_33FC),
            (0x0004_0303, 0x0001_0044),
            (0x1000_0100, 0x0000_13D0),
            (0x0000_8000, 0x0000_33FC),
            (0x0000_8000, 0x0000_33FC),
            (0x1300_0004, 0x0000_0020),
            (0x1302_000D, 0x0000_0024),
            (0x1302_0095, 0x0000_0024),
            (0x0000_0002, 0x0000_2008),
            (0x0000_0000, 0x0000_2008),
            (0x0004_0380, 0x0002_0044),
        ],
    );
    ddr_ctrl_secure_alias_write_table(0x012E, [(0x0004_0380, 0x0002_0044)]);
    ddr_ctrl_secure_alias_write_table(
        0x0180,
        [
            (0x0004_0B43, 0x0000_0044),
            (0x1300_0010, 0x0000_0020),
            (0x0000_0002, 0x0000_2008),
            (0x0000_0002, 0x0000_2008),
            (0x1300_0001, 0x0000_13D0),
            (0x0000_8000, 0x0000_33FC),
            (0x0000_0000, 0x0000_33FC),
            (0x0004_0B43, 0x0001_0044),
            (0x1000_0100, 0x0000_13D0),
            (0x0000_8000, 0x0000_33FC),
            (0x0000_8000, 0x0000_33FC),
            (0x1302_000D, 0x0000_0024),
            (0x1302_0095, 0x0000_0024),
            (0x0000_0002, 0x0000_2008),
            (0x0000_0000, 0x0000_2008),
            (0x0004_0B00, 0x0002_0044),
        ],
    );
}

fn config_ddr_ctrl_timing_table() {
    let ddr_ctrl_ctl0_org = unsafe { DDR_CTRL.read(0x44) };

    let next_offset = ddr_ctrl_secure_alias_write_table(
        0x200,
        [(ddr_ctrl_ctl0_org | (1 << 2) | (1 << 12), 0x0001_0044)],
    );

    let cfg_addrs = [
        0x0048, 0x0054, 0x0058, 0x0060, 0x0064, 0x0148, 0x014C, 0x0200, 0x0204, 0x0208, 0x020C,
        0x0220, 0x0224, 0x02C4, 0x02C0, 0x0380, 0x0384, 0x0388, 0x0080, 0x0A00, 0x0AC0, 0x0ACC,
    ];

    let cfg_table = cfg_addrs.map(|addr| (unsafe { DDR_CTRL.read(addr) }, addr));

    let next_offset = ddr_ctrl_secure_alias_write_table(next_offset, cfg_table);

    let ddr_ctrl_cfg2_offset = DDR_CTRL_CHANNEL_OFFSET + 0x0104;
    let ddr_ctrl_cfg2_org = unsafe { DDR_CTRL_CHANNEL.read(0x0104) };

    let next_offset = unsafe {
        DDR_CTRL_CHANNEL.write([(0x0104, ddr_ctrl_cfg2_org & !(0b1111 << 28))]);
        let next_offset = ddr_ctrl_secure_alias_write_table(
            next_offset,
            [(ddr_ctrl_cfg2_org & !(0b1111 << 28), ddr_ctrl_cfg2_offset)],
        );
        copy_ddr_ctrl_timing_registers(next_offset)
    };

    let next_offset = [0b0101, 0b1010, 0b1111]
        .into_iter()
        .map(|fp_tag| ddr_ctrl_cfg2_org & !(0b1111 << 28) | (fp_tag << 28))
        .map(|fp| (fp, fp & !(0b0011 << 28)))
        .fold(next_offset, |next_offset, (fp, fp_cleared)| unsafe {
            DDR_CTRL_CHANNEL.write([(0x0104, fp)]);
            let next_offset = ddr_ctrl_secure_alias_write_table(
                next_offset,
                [(fp_cleared, ddr_ctrl_cfg2_offset)],
            );
            copy_ddr_ctrl_timing_registers(next_offset)
        });

    ddr_ctrl_secure_alias_write_table(
        next_offset,
        [
            (0x0002_0200, 0x0000_13E0),
            (0x1300_0010, 0x0000_13D0),
            (0x0001_0000, 0x0000_33FC),
            (0x0001_0000, 0x0000_33FC),
            (0x1300_0008, 0x0000_0020),
            (0x1300_0004, 0x0000_0020),
            (0x1302_0000, 0x0000_0028),
            (0x1300_0001, 0x0000_13D0),
            (0x0000_8000, 0x0000_33FC),
            (0x0000_8000, 0x0000_33FC),
            (0x1000_0100, 0x0000_13D0),
            (0x0000_8000, 0x0000_33FC),
            (0x0000_8000, 0x0000_33FC),
            (0x1302_000D, 0x0000_0024),
            (0x1302_0003, 0x0000_0024),
            (ddr_ctrl_ctl0_org, 0x0002_0044),
        ],
    );

    unsafe {
        DDR_PHY.write([
            (0x1_0104, 0x0000_1100),
            (0x1_0108, 0x0001_0000),
            (0x1_0100, 0x0000_0020),
            (0x1_0104, 0x0000_00FF),
            (0x1_0108, 0x0001_001C),
            (0x1_0100, 0x0000_0021),
            (0x1_0104, 0x0000_0000),
            (0x1_0108, 0x0005_001C),
            (0x1_0100, 0x0000_0022),
        ]);

        DDR_CTRL_CHANNEL.write([(0x0104, ddr_ctrl_cfg2_org)]);
    }
}

fn copy_ddr_ctrl_timing_registers(next_offset: u32) -> u32 {
    let timing_registers_addrs = [
        0x0300, 0x030C, 0x0310, 0x0314, 0x038C, 0x0390, 0x0394, 0x0398, 0x039C, 0x03A0, 0x03A4,
        0x03A8, 0x03AC, 0x03B0, 0x03B4, 0x03B8, 0x03BC, 0x03C0, 0x03C4, 0x0400, 0x03D8, 0x034C,
        0x13E4, 0x13EC,
    ];

    let timing_table = timing_registers_addrs.map(|addr| (unsafe { DDR_CTRL.read(addr) }, addr));

    ddr_ctrl_secure_alias_write_table(next_offset, timing_table)
}

fn ddr_ctrl_secure_alias_write_table<const T: usize>(offset: u32, table: [(u32, u32); T]) -> u32 {
    for (index, (data, addr)) in table.into_iter().enumerate() {
        unsafe {
            DDR_CTRL_SECURE_ALIAS.write([
                (0x74, data),
                (0x78, addr),
                (0x70, offset + index as u32),
            ]);
        }
    }
    offset + table.len() as u32
}
