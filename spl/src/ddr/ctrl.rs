use super::{ByteMode, DDR_CTRL, DDR_CTRL_CHANNEL, DDR_CTRL_PHY_CONTROL, DdrCapacity, byte};

pub fn init(byte_mode: ByteMode) {
    config_ctrl_global();
    config_ctrl_channal();
    config_timing();
    if matches!(byte_mode, ByteMode::Enable) {
        byte::set_byte_mode_parameter();
    }
    select_freq();
    config_timing_count();
}

pub fn config_addr_mapping(capacity: DdrCapacity) {
    let (area_length, cs1_start_high, row) = match capacity {
        DdrCapacity::GB16 => (0x11_u32, 2_u32, 8_u32),
        DdrCapacity::GB8 => (0x10_u32, 1_u32, 7_u32),
    };
    let addrmap = (row << 8) | (0x3 << 4) | 0x2;

    unsafe {
        let value_0x00 = DDR_CTRL_CHANNEL.read(0x00);
        let value_0x08 = DDR_CTRL_CHANNEL.read(0x08);
        let value_0x20 = DDR_CTRL_CHANNEL.read(0x20);
        let value_0x24 = DDR_CTRL_CHANNEL.read(0x24);

        DDR_CTRL_CHANNEL.write([
            (0x00, value_0x00 & !(0xFF9F << 16) | (area_length << 16)),
            (0x04, 0x0),
            (0x08, value_0x08 & !(0xFF9F << 16) | (area_length << 16)),
            (0x0C, cs1_start_high),
            (0x20, value_0x20 & !0x0FF3 | addrmap),
            (0x24, value_0x24 & !0x0FF3 | addrmap),
        ]);
    }
}

fn config_ctrl_global() {
    unsafe {
        DDR_CTRL.write([
            (0x0044, 0x0004_0300),
            (0x0048, 0x0000_0001),
            (0x0064, 0x100D_0803),
            (0x0050, 0x0000_00FF),
            (0x0058, 0x3FD5_3FD5),
            (0x0180, 0x0001_0200),
            (0x0080, 0x0000_0000),
            (0x0A00, 0x0000_0000),
            (0x0AC0, 0x0000_0000),
            (0x0ACC, 0xFFFF_FFFF),
        ]);
    }
}

fn config_ctrl_channal() {
    unsafe {
        DDR_CTRL_CHANNEL.write([
            (0x00, 0x0010_0001),
            (0x04, 0x0000_0000),
            (0x08, 0x0010_0001),
            (0x0C, 0x0000_0001),
            (0x20, 0x0503_0732),
            (0x24, 0x0503_0732),
            (0xC0, 0x1400_8000),
            (0xC4, 0x0000_00B8),
            (0xC8, 0x0000_FFFF),
            (0xCC, 0x0000_0000),
        ]);
    }
}

fn config_timing() {
    unsafe {
        // 3200 MT
        DDR_CTRL_CHANNEL.write([
            (0x0104, 0xF080_0400),
            (0x0100, 0x0000_0E20),
            (0x010C, 0x9D19_4314),
            (0x0110, 0x2034_0000),
            (0x0114, 0x2034_0000),
            (0x018C, 0x0000_0030),
            (0x0190, 0x0640_0030),
            (0x0194, 0x80E0_01C0),
            (0x01FC, 0x000C_005E),
            (0x0198, 0x01CC_01CC),
            (0x019C, 0x0018_1818),
            (0x01A0, 0x0818_0C0C),
            (0x01A4, 0x0000_0003),
            (0x01A8, 0x0000_0217),
            (0x01AC, 0x3065_1D44),
            (0x01B0, 0x1120_080F),
            (0x01B4, 0x0800_1000),
            (0x01B8, 0x0000_0C00),
            (0x01BC, 0x0202_0404),
            (0x01C0, 0x1000_0004),
            (0x01C4, 0x0000_0006),
            (0x01D8, 0x0001_0190),
            (0x014C, 0x000C_4090),
        ]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x1500_0A02), (0x03EC, 0x0000_046C)]);

        // 2400 MT
        DDR_CTRL_CHANNEL.write([
            (0x0104, 0xA080_0400),
            (0x0100, 0x0000_0C18),
            (0x010C, 0x9D19_4314),
            (0x0110, 0x0034_0000),
            (0x0114, 0x0034_0000),
            (0x018C, 0x0043_0000),
            (0x0190, 0x0535_0028),
            (0x0194, 0x80A8_0151),
            (0x01FC, 0x000C_005E),
            (0x0198, 0x017F_017F),
            (0x019C, 0x0014_1414),
            (0x01A0, 0x0714_0A0A),
            (0x01A4, 0x0000_0003),
            (0x01A8, 0x0000_0213),
            (0x01AC, 0x3654_1838),
            (0x01B0, 0x1C18_0A18),
            (0x01B4, 0x0800_0E00),
            (0x01B8, 0x0000_0E00),
            (0x01BC, 0x0202_0404),
            (0x01C0, 0x1000_0004),
            (0x01C4, 0x0000_0004),
            (0x01D8, 0x0000_D94E),
            (0x014C, 0x0007_204A),
        ]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x1300_0802), (0x03EC, 0x0000_0450)]);

        // 1600 MT
        DDR_CTRL_CHANNEL.write([
            (0x0104, 0x5080_0400),
            (0x0100, 0x0000_080E),
            (0x010C, 0x9D19_4314),
            (0x0110, 0x0034_0000),
            (0x0114, 0x0034_0000),
            (0x018C, 0x0028_0018),
            (0x0190, 0x0320_0018),
            (0x0194, 0x8070_00E0),
            (0x01FC, 0x000C_005E),
            (0x0198, 0x00E6_00E6),
            (0x019C, 0x000C_0C0C),
            (0x01A0, 0x050C_0606),
            (0x01A4, 0x0000_0003),
            (0x01A8, 0x0000_020C),
            (0x01AC, 0x1833_0F22),
            (0x01B0, 0x110F_080F),
            (0x01B4, 0x0800_0800),
            (0x01B8, 0x0000_0600),
            (0x01BC, 0x0202_0404),
            (0x01C0, 0x0000_0003),
            (0x01C4, 0x0000_0003),
            (0x01D8, 0x0000_8190),
            (0x014C, 0x0003_0848),
        ]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x0A00_0402), (0x03EC, 0x0000_0480)]);

        // 1200 MT
        DDR_CTRL_CHANNEL.write([
            (0x0104, 0x0080_0400),
            (0x0100, 0x0000_080E),
            (0x010C, 0x9D19_4314),
            (0x0110, 0x0034_0000),
            (0x0114, 0x0034_0000),
            (0x018C, 0x0028_0018),
            (0x0190, 0x0320_0018),
            (0x0194, 0x8054_00A8),
            (0x01FC, 0x000C_005E),
            (0x0198, 0x00E6_00E6),
            (0x019C, 0x000C_0C0C),
            (0x01A0, 0x050C_0606),
            (0x01A4, 0x0000_0003),
            (0x01A8, 0x0000_020C),
            (0x01AC, 0x1833_0F22),
            (0x01B0, 0x110F_080F),
            (0x01B4, 0x0800_0800),
            (0x01B8, 0x0000_0600),
            (0x01BC, 0x0202_0404),
            (0x01C0, 0x0000_0002),
            (0x01C4, 0x0000_0003),
            (0x01D8, 0x0000_8190),
            (0x014C, 0x0003_0848),
        ]);
        DDR_CTRL_PHY_CONTROL.write([(0x03E4, 0x0A00_0402), (0x03EC, 0x0000_0480)]);

        DDR_CTRL_CHANNEL.modify([(0x0108, |value| {
            (value & !(0b1111_1111 << 20)) | (0b0001_0000 << 20)
        })]);
    }
}

fn select_freq() {
    unsafe {
        DDR_CTRL_CHANNEL.modify([(0x0104, |value| value & !(0xF << 28))]);
    }
}

fn config_timing_count() {
    unsafe {
        DDR_CTRL_CHANNEL.write([
            (0x0180, 0x0030_D400),
            (0x0184, 0x0004_E200),
            (0x0188, 0x0C80_0000),
        ]);
    }
}
