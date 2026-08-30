//! TWSI8 master sequences.
//!
//! `init` enables the 204.8 MHz gate, PLL1 /5, GPIO118/119 AF2, then APBC-resets
//! TWSI8 and turns its clocks on. `write` left-shifts the 7-bit address and
//! appends the write bit, then issues START / data / STOP per byte while
//! waiting for TX empty and ACK. `reset` soft-resets the unit, programs FAST
//! master, and clears status.

use core::time::Duration;

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{
    APBC, APBS, ClockGating, Control, I2C, MPMU, PINMUX, Pinmux, PllXSw2Control, Status,
    Twsi8ClockResetControl, time,
};

/// Enable 204.8 MHz, PLL1 /5, GPIO118/119 AF2, then APBC-reset and ungate TWSI8.
pub fn init() {
    log::info!("i2c init");
    MPMU.clock_gating.modify(ClockGating::CLK_204P8M::SET);
    APBS.pll1_sw2_control
        .modify(PllXSw2Control::PLL_DIV5_EN::SET);
    PINMUX.gpio_118_i2c.modify(Pinmux::AF_SEL::FUNCTION_2);
    PINMUX.gpio_119_i2c.modify(Pinmux::AF_SEL::FUNCTION_2);
    APBC.twsi8_clock_reset_control
        .write(Twsi8ClockResetControl::RST::SET);
    APBC.twsi8_clock_reset_control.write(
        Twsi8ClockResetControl::RST::SET
            + Twsi8ClockResetControl::FNCLK::SET
            + Twsi8ClockResetControl::APBCLK::SET,
    );
    APBC.twsi8_clock_reset_control
        .write(Twsi8ClockResetControl::FNCLK::SET + Twsi8ClockResetControl::APBCLK::SET);
    time::sleep(Duration::from_micros(10));
}

/// 7-bit master write: address << 1 | write, then START / data / STOP per byte, waiting TX empty and ACK.
pub fn write(addr: u8, data: &[u8]) {
    const TYPE_WRITE: u8 = 0;
    let first = addr << 1 | TYPE_WRITE;
    for (index, byte) in [first].iter().chain(data.iter()).enumerate() {
        while !I2C.status.matches_all(Status::BUS_BUSY::CLEAR) {
            time::sleep(Duration::from_micros(10));
        }
        I2C.control
            .modify(Control::START::CLEAR + Control::STOP::CLEAR);
        I2C.data_buffer.set(*byte as u32);
        if index == 0 {
            I2C.control.modify(Control::START::SET);
        }
        if index == data.len() {
            I2C.control.modify(Control::STOP::SET);
        }
        I2C.control
            .modify(Control::ACKNAK::CLEAR + Control::ARBITRATION_INTERRUPT_ENABLE::CLEAR);
        I2C.control.modify(Control::TRANSFER_BYTE::SET);
        while !I2C.status.matches_all(Status::TX_BUFFER_EMPTY::SET) {
            time::sleep(Duration::from_micros(10));
        }
        I2C.status.modify(Status::TX_BUFFER_EMPTY::SET);
        while !I2C.status.matches_all(Status::ACK_NAK_STATUS::CLEAR) {
            time::sleep(Duration::from_micros(10));
        }
    }
}

/// Soft-reset TWSI8, program FAST master, and clear status.
pub fn reset() {
    I2C.control.modify(Control::ENABLE::CLEAR);
    I2C.control.modify(Control::RESET::SET);
    time::sleep(Duration::from_micros(100));
    I2C.control.modify(Control::ENABLE::CLEAR);
    I2C.slave_address.set(0);
    I2C.control.write({
        use Control::*;
        MODE::FAST
            + BUS_ERROR_INTS_ENABLE::SET
            + RX_INTERRUPT_ENABLE::SET
            + TX_INTERRUPT_ENABLE::SET
            + GENERAL_CALL_DISABLE::SET
            + MASTER_CLOCK_ENABLE::SET
    });
    I2C.status.write({
        use Status::*;
        SLAVE_STOP_DETECTED::SET
            + SLAVE_ADDRESS_DETECTED::SET
            + BUS_ERROR_NO_ACK_NAK::SET
            + GENERAL_CALL_ADDRESS_DETECTED::SET
            + RX_BUFFER_FULL::SET
            + TX_BUFFER_EMPTY::SET
            + ARBITRATION_LOSS_DETECTED::SET
            + BUS_BUSY::SET
            + UNIT_BUSY::SET
            + ACK_NAK_STATUS::SET
            + READ_WRITE_MODE::SET
    });
    I2C.control.modify(Control::ENABLE::SET);
    time::sleep(Duration::from_micros(2));
}
