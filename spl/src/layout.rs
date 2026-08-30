/*
DDR
    0x0001_0000  verify scratch
    0x0008_0000  sbi            1 MiB
    0x0020_0000  kernel         14 MiB
    0x0100_0000  dtb            1 MiB
    0x0400_0000  nvme dma       28 KiB
    0x0800_0000  initramfs      64 MiB
SSD
    2048         sbi            1024 LBA
    4096         kernel         24576 LBA
    28672        dtb            512 LBA
    32768        initramfs      131072 LBA
    163840       rootfs         remainder
*/

pub const SBI: GptPart = GptPart {
    name: "sbi",
    lba_start: 2048,
    lba_max: 1024,
    load_base: 0x0008_0000,
    load_max: 0x0010_0000,
};

pub const KERNEL: GptPart = GptPart {
    name: "kernel",
    lba_start: 4096,
    lba_max: 24576,
    load_base: 0x0020_0000,
    load_max: 0x00E0_0000,
};

pub const DTB: GptPart = GptPart {
    name: "dtb",
    lba_start: 28672,
    lba_max: 512,
    load_base: 0x0100_0000,
    load_max: 0x0010_0000,
};

pub const INITRAMFS: GptPart = GptPart {
    name: "initramfs",
    lba_start: 32768,
    lba_max: 131072,
    load_base: 0x0800_0000,
    load_max: 0x0400_0000,
};

pub const ROOTFS: GptTail = GptTail {
    name: "rootfs",
    start_lba: 163840,
};

pub const GPT_PARTITIONS: &[GptPart] = &[SBI, KERNEL, DTB, INITRAMFS];

pub const DDR_TRAIN_VERIFY_BASE: u64 = 0x0001_0000;
pub const DDR_TRAIN_VERIFY_BYTES: u64 = 512;

pub const NVME_DMA_BASE: u64 = 0x0400_0000;
pub const NVME_ASQ_BASE: u64 = NVME_DMA_BASE;
pub const NVME_ACQ_BASE: u64 = NVME_DMA_BASE + 0x1000;
pub const NVME_IOSQ_BASE: u64 = NVME_DMA_BASE + 0x2000;
pub const NVME_IOCQ_BASE: u64 = NVME_DMA_BASE + 0x3000;
pub const NVME_IDENTIFY_BASE: u64 = NVME_DMA_BASE + 0x4000;
pub const NVME_READ_DMA_BASE: u64 = NVME_DMA_BASE + 0x5000;
pub const NVME_READ_DMA_PRP2: u64 = NVME_DMA_BASE + 0x6000;
pub const NVME_DMA_SIZE: u64 = 0x7000;

#[derive(Clone, Copy)]
pub struct GptPart {
    pub name: &'static str,
    pub lba_start: u64,
    pub lba_max: u64,
    pub load_base: u64,
    pub load_max: u64,
}

impl GptPart {
    pub const fn end_lba(self) -> u64 {
        self.lba_start + self.lba_max - 1
    }

    pub const fn before(self, next: Self) {
        assert!(self.lba_max > 0);
        assert!(self.end_lba() < next.lba_start);
        assert!(self.load_base + self.load_max <= next.load_base);
    }
}

#[derive(Clone, Copy)]
pub struct GptTail {
    pub name: &'static str,
    pub start_lba: u64,
}

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
