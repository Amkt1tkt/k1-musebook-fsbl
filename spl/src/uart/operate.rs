use tock_registers::interfaces::{Readable, Writeable};

use super::{LineStatus, UART};

pub fn write_byte(byte: u8) {
    while !UART.line_status.matches_all(LineStatus::TDRQ::SET) {
        core::hint::spin_loop();
    }
    UART.transmit_holding.set(byte as u32);
}

pub fn flush() {
    while !UART.line_status.matches_all(LineStatus::TEMT::SET) {
        core::hint::spin_loop();
    }
}
