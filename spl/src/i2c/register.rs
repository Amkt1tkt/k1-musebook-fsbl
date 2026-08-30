//! TWSI8 register block at 0xD401D800.

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

/// TWSI8 MMIO window.
pub const I2C: MMIO<I2c> = unsafe { MMIO::base(0xD401_D800) };

register_structs! {
    /// TWSI8 control / status / address / data.
    pub I2c {
        /// Unit control (mode, START/STOP, transfer kick).
        (0x00 => pub control: ReadWrite<u32, Control::Register>),
        /// Transfer / bus status (TX empty, ACK, busy).
        (0x04 => pub status: ReadWrite<u32, Status::Register>),
        /// Own slave address (cleared for master-only use).
        (0x08 => pub slave_address: ReadWrite<u32>),
        /// Next TX byte or received RX byte.
        (0x0C => pub data_buffer: ReadWrite<u32>),
        (0x10 => @END),
    }
}

register_bitfields![u32,
    /// TWSI8 unit control.
    pub Control [
        /// Enable bus-error interrupts.
        BUS_ERROR_INTS_ENABLE OFFSET(22) NUMBITS(1) [],
        /// Ignore general-call addresses.
        GENERAL_CALL_DISABLE OFFSET(21) NUMBITS(1) [],
        /// Enable RX interrupt.
        RX_INTERRUPT_ENABLE OFFSET(20) NUMBITS(1) [],
        /// Enable TX interrupt.
        TX_INTERRUPT_ENABLE OFFSET(19) NUMBITS(1) [],
        /// Enable arbitration-loss interrupt.
        ARBITRATION_INTERRUPT_ENABLE OFFSET(18) NUMBITS(1) [],
        /// Unit enable.
        ENABLE OFFSET(14) NUMBITS(1) [],
        /// Master-mode clock enable.
        MASTER_CLOCK_ENABLE OFFSET(13) NUMBITS(1) [],
        /// Soft reset.
        RESET OFFSET(10) NUMBITS(1) [],
        /// Speed mode (FAST / STANDARD).
        MODE OFFSET(8) NUMBITS(2) [
            FAST = 0b01,
            STANDARD = 0b00,
        ],
        /// Kick a one-byte transfer.
        TRANSFER_BYTE OFFSET(3) NUMBITS(1) [],
        /// ACK (0) or NAK (1) the current byte.
        ACKNAK OFFSET(2) NUMBITS(1) [
            NAK = 1,
            ACK = 0,
        ],
        /// Issue STOP after this byte.
        STOP OFFSET(1) NUMBITS(1) [],
        /// Issue START before this byte.
        START OFFSET(0) NUMBITS(1) [],
    ],
    /// TWSI8 transfer / bus status.
    pub Status [
        /// Slave saw a STOP.
        SLAVE_STOP_DETECTED 24,
        /// Slave address matched.
        SLAVE_ADDRESS_DETECTED 23,
        /// Bus error / missing ACK.
        BUS_ERROR_NO_ACK_NAK 22,
        /// General-call address seen.
        GENERAL_CALL_ADDRESS_DETECTED 21,
        /// RX buffer full.
        RX_BUFFER_FULL 20,
        /// TX buffer empty.
        TX_BUFFER_EMPTY 19,
        /// Lost arbitration.
        ARBITRATION_LOSS_DETECTED 18,
        /// Bus is busy.
        BUS_BUSY 16,
        /// Unit is busy.
        UNIT_BUSY 15,
        /// Last byte was NAK when set.
        ACK_NAK_STATUS 14,
        /// Transfer direction.
        READ_WRITE_MODE 13,
    ]
];
