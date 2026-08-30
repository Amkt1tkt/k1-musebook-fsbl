//! iATU region register block at 0xCAB00000 (region 1 at +0x200).

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::{CTRL_CFG_BASE, CTRL_CFG_END, CTRL_MEM_BASE, CTRL_MEM_END, MMIO};

/// Outbound iATU region 0 (Cfg).
pub const PCIE_C_CTRL_ATU_REGION_CFG: MMIO<PcieCtrlAtu> = unsafe { MMIO::base(0xCAB0_0000) };
/// Outbound iATU region 1 (Mem), +0x200.
pub const PCIE_C_CTRL_ATU_REGION_MEM: MMIO<PcieCtrlAtu> =
    unsafe { MMIO::base(0xCAB0_0000 + 0x200) };

register_structs! {
    /// One iATU region (control, base, limit, target).
    pub PcieCtrlAtu {
        (0x00 => pub region_control_1: ReadWrite<u32, RegionControl1::Register>),
        (0x04 => pub region_control_2: ReadWrite<u32, RegionControl2::Register>),
        (0x08 => pub lower_base_addr: ReadWrite<u32, LowerBaseAddr::Register>),
        (0x0C => pub upper_base_addr: ReadWrite<u32>),
        (0x10 => pub limit_addr: ReadWrite<u32, LimitAddr::Register>),
        (0x14 => pub lower_target_addr: ReadWrite<u32, LowerTargetAddr::Register>),
        (0x18 => pub upper_target_addr: ReadWrite<u32>),
        (0x1C => @END),
    }
}

register_bitfields![u32,
    /// Region TLP type.
    pub RegionControl1 [
        /// TLP type for this region.
        TYPE OFFSET(0) NUMBITS(5) [
            MEM = 0,
            IO = 2,
            CFG0 = 4,
            CFG1 = 5,
        ]
    ],
    /// Region enable.
    pub RegionControl2 [
        /// Enable this iATU region.
        REGION_EN 31,
    ],
    /// CPU-side window base.
    pub LowerBaseAddr [
        /// Window base address.
        ADDR OFFSET(0) NUMBITS(32) [
            CFG = super::super::CTRL_CFG_BASE,
            MEM = super::super::CTRL_MEM_BASE,
        ],
    ],
    /// CPU-side window limit.
    pub LimitAddr [
        /// Window limit address.
        ADDR OFFSET(0) NUMBITS(32) [
            CFG = super::super::CTRL_CFG_END,
            MEM = super::super::CTRL_MEM_END,
        ],
    ],
    /// Translated target address.
    pub LowerTargetAddr [
        /// Target address after translation.
        ADDR OFFSET(0) NUMBITS(32) [
            CFG = 0x0100_0000,
            MEM = super::super::CTRL_MEM_BASE,
        ],
    ],
];
