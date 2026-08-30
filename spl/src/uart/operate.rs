//! Polled 16550 transmit.
//!
//! `write_byte` waits for `TDRQ` then writes THR. `flush` waits for `TEMT`.

use tock_registers::interfaces::{Readable, Writeable};

use super::{LineStatus, UART};

/// Wait for `TDRQ`, then write `byte` to THR.
pub fn write_byte(byte: u8) {
    while !UART.line_status.matches_all(LineStatus::TDRQ::SET) {
        core::hint::spin_loop();
    }
    UART.transmit_holding.set(byte as u32);
}

/// Spin until `TEMT` (holding and shift registers empty).
pub fn flush() {
    while !UART.line_status.matches_all(LineStatus::TEMT::SET) {
        core::hint::spin_loop();
    }
}
