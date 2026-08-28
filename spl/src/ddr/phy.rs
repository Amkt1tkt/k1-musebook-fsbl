use tock_registers::interfaces::ReadWriteable;

use super::{
    APMU, DDR_CTRL_CHANNEL, DDR_PHY, DDR_PHY_OTHER_CONTROL, DDR_PHY_SUB_A, DDR_PHY_SUB_B, DdrFreq,
    DdrPhyLdoControl, DdrPhyPllDiv, dram,
};

pub fn init() {
    config_clock_and_power();
    config_phy_global();
    config_amble();
    config_wr_ds_odt_vref();
    config_rx_ds_odt_vref();
    config_common();
    config_other_control();
}

pub fn config_for_manufacturer(manufacturer: dram::Manufacturer) {
    if matches!(manufacturer, dram::Manufacturer::Hynix) {
        config_for_all_sub_and_freq(0x8, |value| value & !(0xFF << 8) | (0xD8 << 8));
        let freqs = [0xF080_0400, 0xA080_0400, 0x5080_0400, 0x0080_0400];
        freqs.iter().for_each(|freq| unsafe {
            DDR_CTRL_CHANNEL.write([(0x0104, *freq)]);
            DDR_CTRL_CHANNEL.modify([
                (0x0110, |value| value | (0x2 << 28) | (1 << 23)),
                (0x0114, |value| value | (0x2 << 28) | (1 << 23)),
                (0x010C, |value| value & !(0xFF << 24) | (0x19 << 24)),
            ]);
        });
    }
}

fn config_clock_and_power() {
    APMU.ddr_phy_pll_div.modify(DdrPhyPllDiv::BYTE_1::VALUE_0F);
    APMU.ddr_phy_ldo_control
        .modify(DdrPhyLdoControl::BIT_10_11::SET);
}

fn config_phy_global() {
    unsafe {
        DDR_PHY_SUB_A.write([(0x0, 0x0)]);
        DDR_PHY_SUB_B.write([(0x0, 0x0)]);
        DDR_PHY_SUB_A.write([(0x0, 0x1)]);
        DDR_PHY_SUB_B.write([(0x0, 0x1)]);

        DDR_PHY.write([
            (0x0064 + DdrFreq::Mt1200.phy_offset(), 0x4349),
            (0x0064 + DdrFreq::Mt1600.phy_offset(), 0x4349),
            (0x0064 + DdrFreq::Mt2400.phy_offset(), 0x4349),
            (0x0064 + DdrFreq::Mt3200.phy_offset(), 0x4349),
        ]);
    }
}

fn config_amble() {
    config_for_all_sub_and_freq(0x4, |value| {
        value & !(0b1111_0000 << 8) | (0b1010_1000 << 8)
    });
}

fn config_wr_ds_odt_vref() {
    config_for_all_sub_and_freq(0xC, |value| {
        value & !(0b1111_1111 << 8) | (0b1101_1000 << 8)
    });
}

fn config_rx_ds_odt_vref() {
    config_for_all_sub_and_freq(0xC, |value| {
        value & !(0b0011_1111 << 16) | (0b0010_0100 << 16)
    });
    config_for_all_sub_and_freq(0x4, |value| value & !(0xFFFF << 16) | (0x5555 << 16));
}

fn config_common() {
    config_for_all_sub_and_freq(0x14, |value| value & !0x0060_0010 | 0x0060_0000);

    config_for_all_sub_and_freq(0x10, |value| value | (0b0001_0000 << 24));
}

fn config_other_control() {
    unsafe {
        DDR_PHY_SUB_A.write([(0x30, 0x1077)]);
        DDR_PHY_OTHER_CONTROL.write([(0x24, 0x0)]);
        DDR_PHY_OTHER_CONTROL.modify([(0x0, |value| value | 0x1)]);
    }
}

fn config_for_all_sub_and_freq(offset: u32, data_process: fn(u32) -> u32) {
    unsafe {
        let data = data_process(DDR_PHY_SUB_A.read(offset));
        let table = [
            (offset + DdrFreq::Mt1200.phy_offset(), data),
            (offset + DdrFreq::Mt1600.phy_offset(), data),
            (offset + DdrFreq::Mt2400.phy_offset(), data),
            (offset + DdrFreq::Mt3200.phy_offset(), data),
        ];
        DDR_PHY_SUB_A.write(table);
        DDR_PHY_SUB_B.write(table);
    }
}
