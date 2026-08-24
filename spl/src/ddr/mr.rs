use core::time::Duration;

use super::{DDR_CTRL, time};

pub fn init() {
    mr_pulse(0x01);
    mr_pulse(0x02);
    mr_pulse(0x0D);
    mr_pulse(0x03);
    mr_pulse(0x16);
    unsafe {
        DDR_CTRL.write([
            (0x20, 0x1100_2000),
            (0x20, 0x1100_1000),
            (0x20, 0x1200_2000),
            (0x20, 0x1200_1000),
        ]);
    }
    mr_pulse(0x0C);
    mr_pulse(0x0E);
    mr_pulse(0x0B);
    mr_pulse(0x17);
}

pub fn config_for_16gb() {
    mr_pulse(0x95);
}

fn mr_pulse(id: u32) {
    unsafe {
        DDR_CTRL.write([(0x24, 0x1302_0000 | id)]);
    }
    time::sleep(Duration::from_micros(1));
}
