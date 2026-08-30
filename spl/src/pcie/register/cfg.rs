//! Standard Type 1 / Type 0 configuration header fields.

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::CTRL_MEM_BASE;

register_structs! {
    /// PCIe configuration header (Type 0/1 mix used by DBI and NVMe).
    pub PcieCtrlCfg {
        (0x00 => _0x00),
        (0x04 => pub command: ReadWrite<u16, Command::Register>),
        (0x06 => pub status: ReadWrite<u16>),
        (0x08 => pub revision_id: ReadWrite<u8>),
        (0x09 => pub program_interface: ReadWrite<u8>),
        (0x0A => pub class_code: ReadWrite<u16, ClassCode::Register>),
        (0x0C => _0x0c),
        (0x10 => pub bar_0: ReadWrite<u32, Bar0::Register>),
        (0x14 => pub bar_1: ReadWrite<u32, Bar1::Register>),
        (0x18 => pub primary_bus_number: ReadWrite<u8>),
        (0x19 => pub secondary_bus_number: ReadWrite<u8>),
        (0x1A => pub subordinate_bus_number: ReadWrite<u8>),
        (0x1B => pub secondary_latency_timer: ReadWrite<u8>),
        (0x1C => _0x1c),
        (0x3C => pub interrupt_line: ReadWrite<u8>),
        (0x3D => pub interrupt_pin: ReadWrite<u8, InterruptPin::Register>),
        (0x3E => _0x3e),
        (0x7C => pub link_capabilities: ReadWrite<u32, LinkCapabilities::Register>),
        (0x80 => _0x80),
        (0xA0 => pub link_control_2: ReadWrite<u32, LinkControl2::Register>),
        (0xA4 => @END),
    }
}

register_bitfields![u16,
    /// PCI command register.
    pub Command [
        /// I/O space enable.
        IO_ACCESS_ENABLE 0,
        /// Memory space enable.
        MEMORY_ACCESS_ENABLE 1,
        /// Bus master enable.
        BUS_MASTER_ENABLE 2,
        /// SERR# reporting enable.
        SERR_REPORTING_ENABLE 8,
    ],
    /// Class code (base + sub).
    pub ClassCode [
        /// Base class code.
        BASE_CLASS_CODE OFFSET(8) NUMBITS(8) [
            VALUE_06 = 0x06,
        ],
        /// Sub-class code.
        SUB_CLASS_CODE OFFSET(0) NUMBITS(8) [
            VALUE_04 = 0x04,
        ],
    ],
];

register_bitfields![u32,
    /// Packed class-code bytes 2–3.
    pub PcieDbiMagicNumber8 [
        /// Class-code bytes 2–3.
        BYTE_2_3 OFFSET(16) NUMBITS(16) [
            MAGIC_NUMBER_0604 = 0x0604,
        ],
    ],
    /// BAR0: low address and memory type.
    pub Bar0 [
        /// Low 32 bits of the BAR.
        BASE_ADDR OFFSET(0) NUMBITS(32) [
            NVME_LOW = super::super::CTRL_MEM_BASE,
        ],
        /// Memory BAR type.
        MEMORY_TYPE OFFSET(1) NUMBITS(2) [
            RANGE_64_BIT = 0b10,
        ],
    ],
    /// BAR1: high 32 bits of a 64-bit BAR.
    pub Bar1 [
        /// High 32 bits of the BAR.
        BASE_ADDR OFFSET(0) NUMBITS(32) [
            NVME_HIGH = 0x0,
        ],
    ],
    /// Link Capabilities.
    pub LinkCapabilities [
        /// Maximum supported link speed.
        MAX_LINK_SPEED OFFSET(0) NUMBITS(3) [
            GEN1 = 0x1,
            GEN2 = 0x2,
            GEN3 = 0x3,
        ],
    ],
    /// Link Control 2.
    pub LinkControl2 [
        /// Target link speed.
        MAX_LINK_SPEED OFFSET(0) NUMBITS(3) [
            GEN1 = 0x1,
            GEN2 = 0x2,
            GEN3 = 0x3,
        ],
    ],
];

register_bitfields![u8,
    /// Interrupt pin.
    pub InterruptPin [
        FULL OFFSET(0) NUMBITS(8) [
            VALUE_01 = 0x01,
        ],
    ],
];
