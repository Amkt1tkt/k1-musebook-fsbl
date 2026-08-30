//! APB clock/reset (APBC) at `0xD4015000`.
//!
//! This tree only maps TWSI8 functional/APB clocks and reset.

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

/// APBC MMIO window at `0xD4015000`.
pub const APBC: MMIO<Apbc> = unsafe { MMIO::base(0xD401_5000) };

register_structs! {
    /// APBC register block (TWSI8 clock/reset only).
    pub Apbc {
        (0x00 => _0x00),
        (0x20 => pub twsi8_clock_reset_control: ReadWrite<u32, Twsi8ClockResetControl::Register>),
        (0x24 => @END),
    }
}

register_bitfields![u32,
    /// TWSI8 functional clock, APB clock, and reset.
    pub Twsi8ClockResetControl [
        /// Functional Clock Select
        /// - 0x0: 31.5 MHz
        /// - 0x1: 51.2 MHz
        /// - 0x2: 61.44 MHz
        /// - All other values: Reserved, do not use
        FNCLKSEL OFFSET(4) NUMBITS(3) [],
        /// TWSI8 Reset Generation field resets both the APB and functional domain.
        /// - 0: No Reset
        /// - 1: Reset
        RST OFFSET(2) NUMBITS(1) [],
        /// TWSI8 Functional Clock Enable/Disable.
        /// - 0: Clock off
        /// - 1: Clock on
        FNCLK OFFSET(1) NUMBITS(1) [],
        /// TWSI8 APB Bus Clock Enable/Disable.
        /// - 0: Clock off
        /// - 1: Clock on
        APBCLK OFFSET(0) NUMBITS(1) [],
    ]
];
