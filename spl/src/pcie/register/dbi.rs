//! DBI view of this port's config header at 0xCA800000; port logic at +0x700
//! (LTSSM debug, RO_WR_EN, speed change).

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::{MMIO, PcieCtrlCfg};

/// Port C DBI config header.
pub const PCIE_C_CTRL_DBI_CFG: MMIO<PcieCtrlCfg> = unsafe { MMIO::base(0xCA80_0000) };
/// Port C DBI port-logic block (LTSSM debug, RO_WR_EN, speed change).
pub const PCIE_C_CTRL_DBI_PORT_LOGIC: MMIO<PcieCtrlDbiPortLogic> =
    unsafe { MMIO::base(0xCA80_0000 + 0x700) };

register_structs! {
    /// DWC port-logic registers used for LTSSM and DBI write unlock.
    pub PcieCtrlDbiPortLogic {
        (0x000 => _0x000),
        (0x028 => pub pcie_port_debug_0: ReadWrite<u32, PciePortDebug0::Register>),
        (0x02C => _0x02c),
        (0x10C => pub pcie_link_width_speed_control: ReadWrite<u32, PcieLinkWidthSpeedControl::Register>),
        (0x110 => _0x110),
        (0x1BC => pub misc_control_1_off: ReadWrite<u32, MiscControl1Off::Register>),
        (0x1C0 => @END),
    }
}

register_bitfields![u32,
    /// LTSSM debug state.
    pub PciePortDebug0 [
        /// Current LTSSM state.
        LTSSM_STATE OFFSET(0) NUMBITS(5) [
            L0 = 0x11,
        ],
    ],
    /// Misc control 1.
    pub MiscControl1Off [
        /// Allow writes to DBI read-only fields.
        DBI_RO_WR_EN 0,
    ],
    /// Link width/speed control.
    pub PcieLinkWidthSpeedControl [
        /// Force a directed speed change.
        DIRECT_SPEED_CHANGE OFFSET(17) NUMBITS(1) [],
    ],
];
