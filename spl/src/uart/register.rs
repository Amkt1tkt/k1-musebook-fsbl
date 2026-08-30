//! 16550 UART register block at 0xD4017000.

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

/// 16550 UART MMIO window.
pub const UART: MMIO<Uart> = unsafe { MMIO::base(0xD401_7000) };

register_structs! {
    /// 16550-compatible UART (THR + LSR).
    pub Uart {
        /// Transmit Holding Register (THR).
        (0x00 => pub transmit_holding: ReadWrite<u32>),
        (0x04 => _0x04),
        /// Line Status Register (`TEMT` / `TDRQ`).
        (0x14 => pub line_status: ReadWrite<u32, LineStatus::Register>),
        (0x18 => @END),
    }
}

register_bitfields![u32,
    /// UART line status (`TEMT` / `TDRQ`).
    pub LineStatus [
        /// Transmitter Empty.
        /// Set when the Transmit Holding Register and the Transmit Shift Register are both empty. It is cleared when either the Transmit Holding Register or the Transmit Shift Register contains a data character. In FIFO mode, this field is set when the transmit FIFO and the Transmit Shift Register are both empty.
        /// - 0 = There is data in the Transmit Shift Register, the Transmit Holding Register, or the FIFO.
        /// - 1 = All the data in the transmitter has been shifted out.
        TEMT 6,
        /// Transmit Data Request.
        /// This field indicates that the UART is ready to accept a new character for transmission. In addition, this field causes the UART to issue an interrupt to the when the transmit data request interrupt enable is set and generates the DMA request to the DMA controller if DMA requests and FIFO mode are enabled. This field is set when a character is transferred from the Transmit Holding Register into the Transmit Shift Register. This field is cleared with the loading of the Transmit Holding Register. In FIFO mode, this field is set when half of the characters in the FIFO have been loaded into the Transmit Shift Register or the field in the FIFO Control Register has been set. It is cleared when the FIFO has more than half data. If more than 64 characters are loaded into the FIFO, the excess characters are lost.
        /// - 0 = There is data in the holding register or FIFO waiting to be shifted out.
        /// - 1 = The transmit FIFO has half or less than half data.
        TDRQ 5,
    ]
];
