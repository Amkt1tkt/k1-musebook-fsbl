//! OpenSBI-compatible `fw_dynamic_info` version 2.
//!
//! Magic is `OSBI`, `next_addr` is the kernel, `next_mode` is S-mode, `boot_hart` is 0.

use super::KERNEL;

/// `OSBI` little-endian magic expected by OpenSBI `fw_dynamic`.
const FW_DYNAMIC_MAGIC: usize = u32::from_le_bytes(*b"OSBI") as usize;
/// `fw_dynamic_info` structure version.
const FW_DYNAMIC_VERSION: usize = 2;
/// Next-stage privilege: S-mode.
const FW_DYNAMIC_NEXT_MODE_S: usize = 1;
/// OpenSBI option flags (none).
const FW_DYNAMIC_OPTIONS: usize = 0;
/// Boot hartid recorded in the handoff blob.
const FW_DYNAMIC_BOOT_HART: usize = 0;

/// OpenSBI `fw_dynamic_info` passed to SBI in `a2`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FwDynamicInfo {
    /// `OSBI` magic.
    pub magic: usize,
    /// Structure version (2).
    pub version: usize,
    /// Next-stage entry (kernel load address).
    pub next_addr: usize,
    /// Next-stage privilege (`1` = S-mode).
    pub next_mode: usize,
    /// OpenSBI option flags.
    pub options: usize,
    /// Preferred boot hartid.
    pub boot_hart: usize,
}

impl FwDynamicInfo {
    /// Fill the v2 fields from the layout table (`next_addr` = kernel).
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
