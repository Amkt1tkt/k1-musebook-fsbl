pub const KERNEL_PARTITION_NAME: &str = "kernel";
pub const SBI_PARTITION_NAME: &str = "sbi";
pub const DTB_PARTITION_NAME: &str = "dtb";
pub const INITRAMFS_PARTITION_NAME: &str = "initramfs";

pub const SBI_BASE: u64 = 0x0008_0000;
pub const SBI_SIZE: u64 = 0x0010_0000;
pub const KERNEL_BASE: u64 = 0x0020_0000;
pub const KERNEL_SIZE: u64 = 0x00E0_0000;
pub const DTB_BASE: u64 = 0x0100_0000;
pub const DTB_SIZE: u64 = 0x0010_0000;
pub const INITRAMFS_BASE: u64 = 0x0800_0000;
pub const INITRAMFS_SIZE: u64 = 0x0400_0000;

const FW_DYNAMIC_MAGIC: usize = u32::from_le_bytes(*b"OSBI") as usize;
const FW_DYNAMIC_VERSION: usize = 2;
const FW_DYNAMIC_NEXT_MODE_S: usize = 1;
const FW_DYNAMIC_OPTIONS: usize = 0;
const FW_DYNAMIC_BOOT_HART: usize = 0;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FwDynamicInfo {
    pub magic: usize,
    pub version: usize,
    pub next_addr: usize,
    pub next_mode: usize,
    pub options: usize,
    pub boot_hart: usize,
}

impl FwDynamicInfo {
    pub const fn new() -> Self {
        Self {
            magic: FW_DYNAMIC_MAGIC,
            version: FW_DYNAMIC_VERSION,
            next_addr: KERNEL_BASE as usize,
            next_mode: FW_DYNAMIC_NEXT_MODE_S,
            options: FW_DYNAMIC_OPTIONS,
            boot_hart: FW_DYNAMIC_BOOT_HART,
        }
    }
}
