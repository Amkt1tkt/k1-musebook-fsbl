use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

use super::MMIO;

pub const CCI: MMIO<Cci> = unsafe { MMIO::base(0xD850_0000) };

register_structs! {
    pub Cci {
        (0x0000 => _0x0000),
        (0x000C => pub status: ReadOnly<u32, Status::Register>),
        (0x0010 => _0x0010),
        (0x1000 => pub cluster_0_snoop_control: ReadWrite<u32, SnoopControl::Register>),
        (0x1004 => _0x1004),
        (0x2000 => pub cluster_1_snoop_control: ReadWrite<u32, SnoopControl::Register>),
        (0x2004 => @END),
    }
}

register_bitfields![u32,
    pub Status [
        /// A snoop or DVM enable change is still propagating through the interconnect
        CHANGE_PENDING 0,
    ],
    pub SnoopControl [
        /// Enable issuing of DVM message requests from this interface
        DVM_EN 1,
        /// Enable issuing of snoop requests from this interface
        SNOOP_EN 0,
    ],
];
