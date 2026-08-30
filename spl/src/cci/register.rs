//! CCI register block at 0xD8500000.

use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

use super::MMIO;

/// CCI MMIO window (cluster 0 iface @ +0x1000, cluster 1 @ +0x2000).
pub const CCI: MMIO<Cci> = unsafe { MMIO::base(0xD850_0000) };

register_structs! {
    /// ARM CCI status and per-cluster snoop-control registers.
    pub Cci {
        (0x0000 => _0x0000),
        /// Interconnect status (`CHANGE_PENDING`).
        (0x000C => pub status: ReadOnly<u32, Status::Register>),
        (0x0010 => _0x0010),
        /// Cluster 0 snoop + DVM control (+0x1000).
        (0x1000 => pub cluster_0_snoop_control: ReadWrite<u32, SnoopControl::Register>),
        (0x1004 => _0x1004),
        /// Cluster 1 snoop + DVM control (+0x2000).
        (0x2000 => pub cluster_1_snoop_control: ReadWrite<u32, SnoopControl::Register>),
        (0x2004 => @END),
    }
}

register_bitfields![u32,
    /// CCI global status.
    pub Status [
        /// A snoop or DVM enable change is still propagating through the interconnect
        CHANGE_PENDING 0,
    ],
    /// Per-interface snoop / DVM enables.
    pub SnoopControl [
        /// Enable issuing of DVM message requests from this interface
        DVM_EN 1,
        /// Enable issuing of snoop requests from this interface
        SNOOP_EN 0,
    ],
];
