use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::{CTRL_CFG_BASE, CTRL_CFG_END, CTRL_MEM_BASE, CTRL_MEM_END, MMIO};

pub const PCIE_C_CTRL_ATU_REGION_CFG: MMIO<PcieCtrlAtu> = unsafe { MMIO::base(0xCAB0_0000) };
pub const PCIE_C_CTRL_ATU_REGION_MEM: MMIO<PcieCtrlAtu> =
    unsafe { MMIO::base(0xCAB0_0000 + 0x200) };

register_structs! {
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
    pub RegionControl1 [
        TYPE OFFSET(0) NUMBITS(5) [
            MEM = 0,
            IO = 2,
            CFG0 = 4,
            CFG1 = 5,
        ]
    ],
    pub RegionControl2 [
        REGION_EN 31,
    ],
    pub LowerBaseAddr [
        ADDR OFFSET(0) NUMBITS(32) [
            CFG = super::super::CTRL_CFG_BASE,
            MEM = super::super::CTRL_MEM_BASE,
        ],
    ],
    pub LimitAddr [
        ADDR OFFSET(0) NUMBITS(32) [
            CFG = super::super::CTRL_CFG_END,
            MEM = super::super::CTRL_MEM_END,
        ],
    ],
    pub LowerTargetAddr [
        ADDR OFFSET(0) NUMBITS(32) [
            CFG = 0x0100_0000,
            MEM = super::super::CTRL_MEM_BASE,
        ],
    ],
];
