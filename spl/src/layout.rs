//! Single table of SSD GPT partitions and DDR load addresses.
//!
//! Load map: SBI @ 0x80000, kernel @ 0x200000, DTB @ 0x1000000, initramfs @ 0x8000000,
//! NVMe DMA @ 0x4000000, DDR training scratch @ 0x10000.
//! `GptPart` is a named partition with a size cap; `GptTail` (rootfs) takes remaining disk.
//! `GptPart::before` is the compile-time non-overlap check used by `_LAYOUT_VERIFY`.

/// OpenSBI payload: GPT `sbi`, loaded at 0x80000 (cap 1 MiB / 1024 LBA).
pub const SBI: GptPart = GptPart {
    name: "sbi",
    lba_start: 2048,
    lba_max: 1024,
    load_base: 0x0008_0000,
    load_max: 0x0010_0000,
};

/// Kernel image: GPT `kernel`, loaded at 0x200000 (cap 14 MiB / 24576 LBA).
pub const KERNEL: GptPart = GptPart {
    name: "kernel",
    lba_start: 4096,
    lba_max: 24576,
    load_base: 0x0020_0000,
    load_max: 0x00E0_0000,
};

/// Flattened DTB: GPT `dtb`, loaded at 0x1000000 (cap 1 MiB / 512 LBA).
pub const DTB: GptPart = GptPart {
    name: "dtb",
    lba_start: 28672,
    lba_max: 512,
    load_base: 0x0100_0000,
    load_max: 0x0010_0000,
};

/// Initramfs: GPT `initramfs`, loaded at 0x8000000 (cap 64 MiB / 131072 LBA).
pub const INITRAMFS: GptPart = GptPart {
    name: "initramfs",
    lba_start: 32768,
    lba_max: 131072,
    load_base: 0x0800_0000,
    load_max: 0x0400_0000,
};

/// Rootfs tail partition starting at LBA 163840; takes the rest of the disk.
pub const ROOTFS: GptTail = GptTail {
    name: "rootfs",
    start_lba: 163840,
};

/// Named GPT partitions this SPL loads into DDR (excludes `ROOTFS`).
pub const GPT_PARTITIONS: &[GptPart] = &[SBI, KERNEL, DTB, INITRAMFS];

/// DDR training self-check scratch (512 bytes at 0x10000).
pub const DDR_TRAIN_VERIFY_BASE: u64 = 0x0001_0000;
/// Bytes written and read back during DDR training verify.
pub const DDR_TRAIN_VERIFY_BYTES: u64 = 512;

/// NVMe DMA window in DDR (admin/IO queues + identify + read PRP).
pub const NVME_DMA_BASE: u64 = 0x0400_0000;
/// Admin submission queue (4 KiB).
pub const NVME_ASQ_BASE: u64 = NVME_DMA_BASE;
/// Admin completion queue (4 KiB).
pub const NVME_ACQ_BASE: u64 = NVME_DMA_BASE + 0x1000;
/// I/O submission queue (4 KiB).
pub const NVME_IOSQ_BASE: u64 = NVME_DMA_BASE + 0x2000;
/// I/O completion queue (4 KiB).
pub const NVME_IOCQ_BASE: u64 = NVME_DMA_BASE + 0x3000;
/// IDENTIFY data buffer (4 KiB).
pub const NVME_IDENTIFY_BASE: u64 = NVME_DMA_BASE + 0x4000;
/// NVMe read data buffer (4 KiB).
pub const NVME_READ_DMA_BASE: u64 = NVME_DMA_BASE + 0x5000;
/// Second PRP page for reads that span two pages.
pub const NVME_READ_DMA_PRP2: u64 = NVME_DMA_BASE + 0x6000;
/// Bytes reserved for the NVMe DMA window (28 KiB).
pub const NVME_DMA_SIZE: u64 = 0x7000;

/// Named GPT partition with LBA and DDR load caps.
#[derive(Clone, Copy)]
pub struct GptPart {
    /// UTF-16 GPT partition name this SPL matches.
    pub name: &'static str,
    /// First LBA of the partition.
    pub lba_start: u64,
    /// Max LBAs this partition may occupy (not necessarily the on-disk size).
    pub lba_max: u64,
    /// DDR address the image is copied to.
    pub load_base: u64,
    /// Max bytes this SPL will copy into DDR.
    pub load_max: u64,
}

impl GptPart {
    /// Last LBA this partition may use (`lba_start + lba_max - 1`).
    pub const fn end_lba(self) -> u64 {
        self.lba_start + self.lba_max - 1
    }

    /// Compile-time assert that `self` does not overlap `next` on disk or in DDR.
    pub const fn before(self, next: Self) {
        assert!(self.lba_max > 0);
        assert!(self.end_lba() < next.lba_start);
        assert!(self.load_base + self.load_max <= next.load_base);
    }
}

/// Named GPT tail that starts at `start_lba` and takes the remaining disk.
#[derive(Clone, Copy)]
pub struct GptTail {
    /// UTF-16 GPT partition name.
    pub name: &'static str,
    /// First LBA; the end is the last usable LBA.
    pub start_lba: u64,
}

/// Compile-time checks that GPT LBA ranges and DDR windows do not overlap.
const _LAYOUT_VERIFY: () = {
    SBI.before(KERNEL);
    KERNEL.before(DTB);
    DTB.before(INITRAMFS);
    assert!(INITRAMFS.lba_max > 0);
    assert!(GPT_PARTITIONS[0].lba_start >= 34);
    assert!(INITRAMFS.end_lba() < ROOTFS.start_lba);
    assert!(DDR_TRAIN_VERIFY_BASE + DDR_TRAIN_VERIFY_BYTES <= SBI.load_base);
    assert!(DTB.load_base + DTB.load_max <= NVME_DMA_BASE);
    assert!(NVME_DMA_BASE + NVME_DMA_SIZE <= INITRAMFS.load_base);
};
