use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

pub const GENERIC_COUNTER: MMIO<GenericCounter> = unsafe { MMIO::base(0xD500_1000) };

register_structs! {
    pub GenericCounter {
        (0x00 => pub control: ReadWrite<u32, Control::Register>),
        (0x04 => pub status: ReadWrite<u32, Status::Register>),
        /// Value of counter [31:0]
        (0x08 => pub value_low: ReadWrite<u32>),
        /// Value of counter [63:32]
        (0x0C => pub value_high: ReadWrite<u32>),
        (0x10 => _0x10),
        /// Frequency in number of ticks per second
        (0x20 => pub ticks_per_second: ReadWrite<u32>),
        (0x24 => @END),
    }
}

register_bitfields![u32,
    pub Control [
        /// Halt on debug.
        /// The possible values are:
        /// - 0: HLTDBG signal into the counter has no effect
        /// - 1: HLTDBG signal into the counter halts the counter
        HDBG 1,
        /// Enable/Disable counter.
        /// The possible values are:
        /// - 0: The counter is disabled and not incrementing
        /// - 1: The counter is enabled and is incrementing
        EN 0,
    ],
    Status [
        /// Debug halted
        HDBG 1,
    ]
];
