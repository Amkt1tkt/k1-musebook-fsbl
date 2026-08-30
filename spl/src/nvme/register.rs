//! NVMe 1.x controller registers (CAP/CC/CSTS/AQA/ASQ/ACQ); doorbells at +0x1000.

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::{MMIO, NVME_ACQ_BASE, NVME_ASQ_BASE, NVME_CTRL_BASE};

/// NVMe controller MMIO.
pub const NVME_CTRL: MMIO<NvmeCtrl> = unsafe { MMIO::base(NVME_CTRL_BASE) };

/// Doorbell register base (stride from CAP.DSTRD).
pub const NVME_DOORBELL_BASE: u32 = NVME_CTRL_BASE + 0x1000;

/// Admin SQ depth.
pub const ASQ_SIZE: u32 = 64;
/// Admin CQ depth.
pub const ACQ_SIZE: u32 = 64;
/// I/O SQ depth.
pub const IOSQ_SIZE: u32 = 64;
/// I/O CQ depth.
pub const IOCQ_SIZE: u32 = 64;

/// Admin queue pair ID.
pub const ADMIN_QID: u32 = 0;
/// I/O queue pair ID.
pub const IO_QID: u32 = 1;

register_structs! {
    /// NVMe controller register map.
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
    /// Controller Configuration (CC).
    pub Config [
        /// Controller enable.
        ENABLE OFFSET(0) NUMBITS(1) [],
        /// NVM command set selected.
        CSS_NVM OFFSET(4) NUMBITS(1) [],
        /// 4 KiB memory page size.
        MPS_4K OFFSET(7) NUMBITS(1) [],
        /// Round-robin arbitration.
        AMS_RR OFFSET(11) NUMBITS(1) [],
        /// Normal shutdown notification.
        SHN_NORMAL OFFSET(14) NUMBITS(1) [],
        /// I/O SQ entry size (2^n bytes).
        IOSQ_ENTRY_SIZE OFFSET(16) NUMBITS(4) [
            BYTES_64 = 64_usize.ilog2(),
        ],
        /// I/O CQ entry size (2^n bytes).
        IOCQ_ENTRY_SIZE OFFSET(20) NUMBITS(4) [
            BYTES_16 = 16_usize.ilog2(),
        ],
    ],
    /// Controller Status (CSTS).
    pub Status [
        /// Controller ready.
        READY 0,
    ],
    /// Admin Queue Attributes (AQA).
    pub Aqa [
        /// Admin SQ size (entries − 1).
        ASQ_SIZE OFFSET(0) NUMBITS(16) [
            ENTRY_64 = super::super::ASQ_SIZE - 1,
        ],
        /// Admin CQ size (entries − 1).
        ACQ_SIZE OFFSET(16) NUMBITS(16) [
            ENTRY_64 = super::super::ACQ_SIZE - 1,
        ],
    ],

];

register_bitfields![u64,
    /// Controller Capabilities (CAP).
    pub Cap [
        /// Doorbell stride (2^(2+DSTRD) bytes).
        DSTRD OFFSET(32) NUMBITS(4) [],
    ],
    /// Admin Submission Queue Base (ASQ).
    pub AsqBase [
        /// Admin SQ base address.
        ADDR OFFSET(0) NUMBITS(32) [
            ASQ_BASE = super::super::NVME_ASQ_BASE,
        ],
    ],
    /// Admin Completion Queue Base (ACQ).
    pub AcqBase [
        /// Admin CQ base address.
        ADDR OFFSET(0) NUMBITS(32) [
            ACQ_BASE = super::super::NVME_ACQ_BASE,
        ],
    ],
];
