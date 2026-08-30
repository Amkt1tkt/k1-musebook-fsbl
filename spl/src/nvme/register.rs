use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::{MMIO, NVME_ACQ_BASE, NVME_ASQ_BASE, NVME_CTRL_BASE};

pub const NVME_CTRL: MMIO<NvmeCtrl> = unsafe { MMIO::base(NVME_CTRL_BASE) };

pub const NVME_DOORBELL_BASE: u32 = NVME_CTRL_BASE + 0x1000;

pub const ASQ_SIZE: u32 = 64;
pub const ACQ_SIZE: u32 = 64;
pub const IOSQ_SIZE: u32 = 64;
pub const IOCQ_SIZE: u32 = 64;

pub const ADMIN_QID: u32 = 0;
pub const IO_QID: u32 = 1;

register_structs! {
    pub NvmeCtrl {
        (0x00 => pub cap: ReadWrite<u64, Cap::Register>),
        (0x08 => _0x08),
        (0x14 => pub config: ReadWrite<u32, Config::Register>),
        (0x18 => _0x18),
        (0x1C => pub status: ReadWrite<u32, Status::Register>),
        (0x20 => _0x20),
        (0x24 => pub aqa: ReadWrite<u32, Aqa::Register>),
        (0x28 => pub asq_base: ReadWrite<u64, AsqBase::Register>),
        (0x30 => pub acq_base: ReadWrite<u64, AcqBase::Register>),
        (0x38 => @END),
    }
}

register_bitfields![u32,
    pub Config [
        ENABLE OFFSET(0) NUMBITS(1) [],
        CSS_NVM OFFSET(4) NUMBITS(1) [],
        MPS_4K OFFSET(7) NUMBITS(1) [],
        AMS_RR OFFSET(11) NUMBITS(1) [],
        SHN_NORMAL OFFSET(14) NUMBITS(1) [],
        IOSQ_ENTRY_SIZE OFFSET(16) NUMBITS(4) [
            BYTES_64 = 64_usize.ilog2(),
        ],
        IOCQ_ENTRY_SIZE OFFSET(20) NUMBITS(4) [
            BYTES_16 = 16_usize.ilog2(),
        ],
    ],
    pub Status [
        READY 0,
    ],
    pub Aqa [
        ASQ_SIZE OFFSET(0) NUMBITS(16) [
            ENTRY_64 = super::super::ASQ_SIZE - 1,
        ],
        ACQ_SIZE OFFSET(16) NUMBITS(16) [
            ENTRY_64 = super::super::ACQ_SIZE - 1,
        ],
    ],

];

register_bitfields![u64,
    pub Cap [
        DSTRD OFFSET(32) NUMBITS(4) [],
    ],
    pub AsqBase [
        ADDR OFFSET(0) NUMBITS(32) [
            ASQ_BASE = super::super::NVME_ASQ_BASE,
        ],
    ],
    pub AcqBase [
        ADDR OFFSET(0) NUMBITS(32) [
            ACQ_BASE = super::super::NVME_ACQ_BASE,
        ],
    ],
];
