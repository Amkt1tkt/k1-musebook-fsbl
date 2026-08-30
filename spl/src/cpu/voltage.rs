//! Raise `VDD_CORE` to 1.05 V via I2C8.
//!
//! Writes SPM8821 (address 0x41) BUCK1 VSEL = 0x6E. Without this, 1.6 GHz
//! undervolts the core. After the write, I2C is reset and the rail settles
//! for 2 ms.

use core::time::Duration;

use super::{i2c, time};

/// SPM8821 7-bit I2C address.
const PMIC_I2C_ADDR: u8 = 0x41;
/// SPM8821 BUCK1 voltage-select register.
const SPM8821_BUCK1_VSEL_REG: u8 = 0x48;
/// BUCK1 VSEL encoding for 1.05 V.
const VSEL_1V05: u8 = 0x6E;

/// Write SPM8821 (0x41) BUCK1 VSEL=0x6E over I2C8, reset I2C, then wait 2 ms.
pub fn raise_voltage() {
    log::info!("raise cpu voltage to 1.05V");
    i2c::reset();
    i2c::write(PMIC_I2C_ADDR, &[SPM8821_BUCK1_VSEL_REG, VSEL_1V05]);
    i2c::reset();
    time::sleep(Duration::from_millis(2));
}
