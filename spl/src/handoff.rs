use super::KERNEL;

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
            next_addr: KERNEL.load_base as usize,
            next_mode: FW_DYNAMIC_NEXT_MODE_S,
            options: FW_DYNAMIC_OPTIONS,
            boot_hart: FW_DYNAMIC_BOOT_HART,
        }
    }
}
