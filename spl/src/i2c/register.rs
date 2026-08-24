use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

pub const I2C: MMIO<I2c> = unsafe { MMIO::base(0xD401_D800) };

register_structs! {
    pub I2c {
        (0x00 => pub control: ReadWrite<u32, Control::Register>),
        (0x04 => pub status: ReadWrite<u32, Status::Register>),
        (0x08 => pub slave_address: ReadWrite<u32>),
        (0x0C => pub data_buffer: ReadWrite<u32>),
        (0x10 => @END),
    }
}

register_bitfields![u32,
    pub Control [
        BUS_ERROR_INTS_ENABLE OFFSET(22) NUMBITS(1) [],
        GENERAL_CALL_DISABLE OFFSET(21) NUMBITS(1) [],
        RX_INTERRUPT_ENABLE OFFSET(20) NUMBITS(1) [],
        TX_INTERRUPT_ENABLE OFFSET(19) NUMBITS(1) [],
        ARBITRATION_INTERRUPT_ENABLE OFFSET(18) NUMBITS(1) [],
        ENABLE OFFSET(14) NUMBITS(1) [],
        MASTER_CLOCK_ENABLE OFFSET(13) NUMBITS(1) [],
        RESET OFFSET(10) NUMBITS(1) [],
        MODE OFFSET(8) NUMBITS(2) [
            FAST = 0b01,
            STANDARD = 0b00,
        ],
        TRANSFER_BYTE OFFSET(3) NUMBITS(1) [],
        ACKNAK OFFSET(2) NUMBITS(1) [
            NAK = 1,
            ACK = 0,
        ],
        STOP OFFSET(1) NUMBITS(1) [],
        START OFFSET(0) NUMBITS(1) [],
    ],
    pub Status [
        SLAVE_STOP_DETECTED 24,
        SLAVE_ADDRESS_DETECTED 23,
        BUS_ERROR_NO_ACK_NAK 22,
        GENERAL_CALL_ADDRESS_DETECTED 21,
        RX_BUFFER_FULL 20,
        TX_BUFFER_EMPTY 19,
        ARBITRATION_LOSS_DETECTED 18,
        BUS_BUSY 16,
        UNIT_BUSY 15,
        ACK_NAK_STATUS 14,
        READ_WRITE_MODE 13,
    ]
];
