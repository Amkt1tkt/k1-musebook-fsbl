use core::time::Duration;

use super::{i2c, time};

const PMIC_I2C_ADDR: u8 = 0x41;
const SPM8821_BUCK1_VSEL_REG: u8 = 0x48;
const VSEL_1V05: u8 = 0x6E;

pub fn raise_voltage() {
    log::info!("raise cpu voltage to 1.05V");
    i2c::reset();
    i2c::write(PMIC_I2C_ADDR, &[SPM8821_BUCK1_VSEL_REG, VSEL_1V05]);
    i2c::reset();
    time::sleep(Duration::from_millis(2));
}
